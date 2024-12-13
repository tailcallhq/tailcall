use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

use anyhow::anyhow;
use markdown::mdast::Node;
use markdown::ParseOptions;
use tailcall::core::config::Source;
use tailcall::core::FileIO;

use crate::file::NativeFileTest;

#[derive(Clone)]
pub struct ExecutionSpec {
    pub env: Option<HashMap<String, String>>,
    pub configs: ConfigHolder,

    // if this is set to true,
    // then we will assert Config<Resolved>
    // instead of asserting the generated config
    pub debug_assert_config: bool,
}

pub struct IO {
    pub fs: NativeFileTest,
    pub paths: Vec<String>,
}

#[derive(Clone)]
pub struct ConfigHolder {
    configs: Vec<(Source, String)>,
}

impl ConfigHolder {
    pub async fn into_io(self) -> IO {
        let fs = NativeFileTest::default();
        let mut paths = vec![];
        for (i, (source, content)) in self.configs.iter().enumerate() {
            let path = format!("config{}.{}", i, source.ext());
            fs.write(&path, content.as_bytes()).await.unwrap();
            paths.push(path);
        }
        IO { fs, paths }
    }
}

impl ExecutionSpec {
    pub fn from_source(path: &Path, contents: String) -> anyhow::Result<Self> {
        let ast = markdown::to_mdast(&contents, &ParseOptions::default()).unwrap();
        let children = ast
            .children()
            .unwrap_or_else(|| panic!("Failed to parse {:?}: empty file unexpected", path))
            .iter()
            .peekable();

        let mut env = None;
        let mut debug_assert_config = false;
        let mut configs = vec![];

        for node in children {
            match node {
                Node::Heading(heading) => {
                    if heading.depth == 2 {
                        if let Some(Node::Text(expect)) = heading.children.first() {
                            let split = expect.value.splitn(2, ':').collect::<Vec<&str>>();
                            match split[..] {
                                [a, b] => {
                                    debug_assert_config =
                                        a.contains("debug_assert") && b.ends_with("true");
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
                    }
                }
                Node::Code(code) => {
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
                                configs.push((source, content));
                            }
                            "env" => {
                                let vars: HashMap<String, String> = match source {
                                    Source::Json => Ok(serde_json::from_str(&content)?),
                                    Source::Yml => Ok(serde_yaml_ng::from_str(&content)?),
                                    _ => Err(anyhow!("Unexpected language in env block in {:?} (only JSON and YAML are supported)", path)),
                                }?;

                                env = Some(vars);
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
                }
                _ => return Err(anyhow!("Unexpected node in {:?}: {:#?}", path, node)),
            }
        }

        Ok(Self { env, configs: ConfigHolder { configs }, debug_assert_config })
    }
}
