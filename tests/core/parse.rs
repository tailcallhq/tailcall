extern crate core;

use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::panic;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use async_graphql_value::ConstValue;
use markdown::mdast::Node;
use markdown::ParseOptions;
use tailcall::cli::javascript;
use tailcall::core::app_context::AppContext;
use tailcall::core::blueprint::Blueprint;
use tailcall::core::cache::InMemoryCache;
use tailcall::core::config::{ConfigModule, Source};
use tailcall::core::runtime::TargetRuntime;
use tailcall::core::worker::{Command, Event};
use tailcall::core::{EnvIO, WorkerIO};

use super::file::File;
use super::http::Http;
use super::model::*;
use super::runtime::ExecutionSpec;

struct Env {
    env: HashMap<String, String>,
}

impl EnvIO for Env {
    fn get(&self, key: &str) -> Option<Cow<'_, str>> {
        self.env.get(key).map(Cow::from)
    }
}

impl Env {
    pub fn init(map: HashMap<String, String>) -> Self {
        Self { env: map }
    }
}

impl ExecutionSpec {
    pub async fn from_source(path: &Path, contents: String) -> anyhow::Result<Self> {
        let ast = markdown::to_mdast(&contents, &ParseOptions::default()).unwrap();
        let mut children = ast
            .children()
            .unwrap_or_else(|| panic!("Failed to parse {:?}: empty file unexpected", path))
            .iter()
            .peekable();

        let mut name: Option<String> = None;
        let mut server: Vec<(Source, String)> = Vec::with_capacity(2);
        let mut mock: Option<Vec<Mock>> = None;
        let mut env: Option<HashMap<String, String>> = None;
        let mut files: BTreeMap<String, String> = BTreeMap::new();
        let mut test: Option<Vec<APIRequest>> = None;
        let mut runner: Option<Annotation> = None;
        let mut check_identity = false;
        let mut sdl_error = false;

        while let Some(node) = children.next() {
            match node {
                Node::Heading(heading) => {
                    if heading.depth == 1 {
                        // Parse test name
                        if name.is_none() {
                            if let Some(Node::Text(text)) = heading.children.first() {
                                name = Some(text.value.clone());
                            } else {
                                return Err(anyhow!(
                                    "Unexpected content of level 1 heading in {:?}: {:#?}",
                                    path,
                                    heading
                                ));
                            }
                        } else {
                            return Err(anyhow!(
                                "Unexpected double-declaration of test name in {:?}",
                                path
                            ));
                        }

                        // Consume optional test description
                        if let Some(Node::Paragraph(_)) = children.peek() {
                            let _ = children.next();
                        }
                    } else if heading.depth == 2 {
                        // TODO: use frontmatter parsing instead of handle it as heading?
                        if let Some(Node::Text(expect)) = heading.children.first() {
                            let split = expect.value.splitn(2, ':').collect::<Vec<&str>>();
                            match split[..] {
                                [a, b] => {
                                    check_identity = a.contains("identity") && b.ends_with("true");
                                    sdl_error = a.contains("error") && b.ends_with("true");
                                    if a.contains("skip") && b.ends_with("true") {
                                        runner = Some(Annotation::Skip);
                                    }
                                }
                                _ => {
                                    return Err(anyhow!(
                                        "Unexpected header annotation {:?} in {:?}",
                                        expect.value,
                                        path,
                                    ))
                                }
                            }
                        }
                    } else if heading.depth == 5 {
                        // Parse annotation
                        return if runner.is_none() {
                            if let Some(Node::Text(text)) = heading.children.first() {
                                Err(anyhow!(
                                    "Unexpected runner annotation {:?} in {:?}",
                                    text.value,
                                    path,
                                ))
                            } else {
                                Err(anyhow!(
                                    "Unexpected content of level 5 heading in {:?}: {:#?}",
                                    path,
                                    heading
                                ))
                            }
                        } else {
                            Err(anyhow!(
                                "Unexpected double-declaration of runner annotation in {:?}",
                                path
                            ))
                        };
                    } else if heading.depth == 4 {
                    } else {
                        return Err(anyhow!(
                            "Unexpected level {} heading in {:?}: {:#?}",
                            heading.depth,
                            path,
                            heading
                        ));
                    }
                }
                Node::Code(code) => {
                    // Parse following code block
                    let (content, lang, meta) = {
                        (
                            code.value.to_owned(),
                            code.lang.to_owned(),
                            code.meta.to_owned(),
                        )
                    };
                    if let Some(meta_str) = meta.as_ref().filter(|s| s.contains('@')) {
                        let temp_cleaned_meta = meta_str.replace('@', "");
                        let name: &str = &temp_cleaned_meta;
                        if let Some(name) = name.strip_prefix("file:") {
                            if files.insert(name.to_string(), content).is_some() {
                                return Err(anyhow!(
                                    "Double declaration of file {:?} in {:#?}",
                                    name,
                                    path
                                ));
                            }
                        } else {
                            let lang = match lang {
                                Some(x) => Ok(x),
                                None => Err(anyhow!(
                                    "Unexpected code block with no specific language in {:?}",
                                    path
                                )),
                            }?;

                            let source = Source::from_str(&lang)?;

                            match name {
                                "config" => {
                                    // Server configs are only parsed if the test isn't skipped.
                                    server.push((source, content));
                                }
                                "mock" => {
                                    if mock.is_none() {
                                        mock = match source {
                                            Source::Json => Ok(serde_json::from_str(&content)?),
                                            Source::Yml => Ok(serde_yaml::from_str(&content)?),
                                            _ => Err(anyhow!("Unexpected language in mock block in {:?} (only JSON and YAML are supported)", path)),
                                        }?;
                                    } else {
                                        return Err(anyhow!("Unexpected number of mock blocks in {:?} (only one is allowed)", path));
                                    }
                                }
                                "env" => {
                                    if env.is_none() {
                                        env = match source {
                                            Source::Json => Ok(serde_json::from_str(&content)?),
                                            Source::Yml => Ok(serde_yaml::from_str(&content)?),
                                            _ => Err(anyhow!("Unexpected language in env block in {:?} (only JSON and YAML are supported)", path)),
                                        }?;
                                    } else {
                                        return Err(anyhow!("Unexpected number of env blocks in {:?} (only one is allowed)", path));
                                    }
                                }
                                "test" => {
                                    if test.is_none() {
                                        test = match source {
                                            Source::Json => Ok(serde_json::from_str(&content)?),
                                            Source::Yml => Ok(serde_yaml::from_str(&content)?),
                                            _ => Err(anyhow!("Unexpected language in test block in {:?} (only JSON and YAML are supported)", path)),
                                        }?;
                                    } else {
                                        return Err(anyhow!("Unexpected number of test blocks in {:?} (only one is allowed)", path));
                                    }
                                }
                                _ => {
                                    return Err(anyhow!(
                                        "Unexpected component {:?} in {:?}: {:#?}",
                                        name,
                                        path,
                                        meta
                                    ));
                                }
                            }
                        }
                    } else {
                        return Err(anyhow!(
                            "Unexpected content of code in {:?}: {:#?}",
                            path,
                            meta
                        ));
                    }
                }
                Node::Definition(d) => {
                    if let Some(title) = &d.title {
                        tracing::info!("Comment found in: {:?} with title: {}", path, title);
                    }
                }
                Node::ThematicBreak(_) => {
                    // skip this for and put execute logic in heading.depth
                    // above to escape ThematicBreaks like
                    // `---`, `***` or `___`
                }
                _ => return Err(anyhow!("Unexpected node in {:?}: {:#?}", path, node)),
            }
        }

        if server.is_empty() {
            return Err(anyhow!(
                "Unexpected blocks in {:?}: You must define a GraphQL Config in an execution test.",
                path,
            ));
        }

        let spec = ExecutionSpec {
            path: path.to_owned(),
            name: name.unwrap_or_else(|| path.file_name().unwrap().to_str().unwrap().to_string()),
            safe_name: path.file_name().unwrap().to_str().unwrap().to_string(),

            server,
            mock,
            env,
            test,
            files,

            runner,

            check_identity,
            sdl_error,
        };

        anyhow::Ok(spec)
    }

    pub async fn app_context(
        &self,
        config: &ConfigModule,
        env: HashMap<String, String>,
        http: Arc<Http>,
    ) -> Arc<AppContext> {
        let blueprint = Blueprint::try_from(config).unwrap();
        let script = blueprint.server.script.clone();

        let http2_only = http.clone();

        let http_worker: Option<Arc<dyn WorkerIO<Event, Command>>> =
            if let Some(script) = script.clone() {
                Some(javascript::init_worker_io(script))
            } else {
                None
            };

        let worker: Option<Arc<dyn WorkerIO<ConstValue, ConstValue>>> = if let Some(script) = script
        {
            Some(javascript::init_worker_io(script))
        } else {
            None
        };

        let runtime = TargetRuntime {
            http,
            http2_only,
            file: Arc::new(File::new(self.clone())),
            env: Arc::new(Env::init(env)),
            cache: Arc::new(InMemoryCache::new()),
            extensions: Arc::new(vec![]),
            cmd_worker: http_worker,
            worker,
        };

        let endpoints = config
            .extensions()
            .endpoint_set
            .clone()
            .into_checked(&blueprint, runtime.clone())
            .await
            .unwrap();

        Arc::new(AppContext::new(blueprint, runtime, endpoints))
    }
}
