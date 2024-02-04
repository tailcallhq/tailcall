use std::collections::HashSet;
use std::fs::{self, canonicalize, read_dir, read_to_string, write, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use async_graphql::parser::types::TypeSystemDefinition;
use testconv::http::{ConfigSource, HttpSpec};
use tailcall::blueprint::{Blueprint, Upstream};
use tailcall::cli::init_runtime;
use tailcall::config::reader::ConfigReader;
use tailcall::config::{Config, ConfigSet};
use tailcall::directive::DirectiveCodec;
use tailcall::print_schema::print_schema;
use tailcall::valid::Validator as _;

use testconv::common::{APIRequest, Annotation, SDLError};

const TEST_ANNOTATION_MSG: &str = "**This test had an assertion with a fail annotation that testconv cannot convert losslessly.** If you need the original responses, you can find it in git history. (For example, at commit [1c32ca9](https://github.com/tailcallhq/tailcall/tree/1c32ca9e8080ae3b17e9cf41078d028d3e0289da))";
const BAD_GRAPHQL_MSG: &str = "This test has invalid GraphQL that wasn't caught by http_spec before conversion. It is skipped right now, but it should be fixed at some point.";

#[inline]
fn is_path_file_ext(path: &Path, ext: &str) -> bool {
    path.is_file()
        && path
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            .as_str()
            == ext
}

fn graphql_iter_spec_part(spec: &str) -> impl Iterator<Item = (String, String)> + '_ {
    spec.split("\n#> ").skip(1).map(|part| {
        let (typ, content) = part.split_once('\n').unwrap();

        (typ.trim().to_string(), content.trim().to_string())
    })
}

async fn generate_client_snapshot(file_stem: &str, config: &ConfigSet) {
    let snapshots_dir =
        canonicalize(PathBuf::from("tests/snapshots")).expect("Could not find snapshots directory");

    let client = print_schema((Blueprint::try_from(config).unwrap()).to_schema());

    let snap = format!(
        "---\nsource: tests/execution_spec.rs\nexpression: client\n---\n{}\n",
        client
    );

    let target = snapshots_dir.join(PathBuf::from(format!(
        "execution_spec__{}.md_client.snap",
        file_stem,
    )));

    write(target, snap).unwrap();
}

async fn generate_client_snapshot_sdl(file_stem: &str, sdl: &str, reader: &ConfigReader) {
    let config = Config::from_sdl(sdl).to_result().unwrap();
    let config = reader.resolve(config).await.unwrap();
    generate_client_snapshot(file_stem, &config).await
}

async fn generate_merged_snapshot(file_stem: &str, config: &Config) {
    let snapshots_dir =
        canonicalize(PathBuf::from("tests/snapshots")).expect("Could not find snapshots directory");

    let merged = Config::default().merge_right(config).to_sdl();

    let snap = format!(
        "---\nsource: tests/execution_spec.rs\nexpression: merged\n---\n{}\n",
        merged,
    );

    let target = snapshots_dir.join(PathBuf::from(format!(
        "execution_spec__{}.md_merged.snap",
        file_stem,
    )));

    write(target, snap).unwrap();
}

async fn generate_merged_snapshot_sdl(file_stem: &str, sdl: &str) {
    let config = Config::from_sdl(sdl).to_result().unwrap();
    generate_merged_snapshot(file_stem, &config).await
}

#[tokio::main]
async fn main() {
    let http_dir =
        canonicalize(PathBuf::from("tests/http")).expect("Could not find http directory");

    let merge_dir = canonicalize(PathBuf::from("tests/graphql/merge"))
        .expect("Could not find graphql/merge directory");

    let client_dir =
        canonicalize(PathBuf::from("tests/graphql")).expect("Could not find graphql directory");

    let errors_dir = canonicalize(PathBuf::from("tests/graphql/errors"))
        .expect("Could not find graphql/errors directory");

    let execution_dir =
        canonicalize(PathBuf::from("tests/execution")).expect("Could not find execution directory");

    let snapshots_dir =
        canonicalize(PathBuf::from("tests/snapshots")).expect("Could not find snapshots directory");

    let mut files_already_processed: HashSet<String> = HashSet::new();

    let reader = ConfigReader::init(init_runtime(&Upstream::default(), None));

    for x in read_dir(http_dir).expect("Could not read http directory") {
        let x = x.unwrap();

        let path = x.path();
        let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();

        if files_already_processed.contains(&file_stem) {
            panic!("File name collision: {}", file_stem);
        }

        if is_path_file_ext(&path, "yml") {
            let f = File::open(&path).expect("Failed to open http spec");

            let old = serde_yaml::from_reader::<File, HttpSpec>(f).unwrap();

            let has_fail_annotation = matches!(old.runner, Some(Annotation::Fail));
            let bad_graphql_skip: bool = match &old.config {
                ConfigSource::File(x) => reader.read(x).await.is_err(),
                ConfigSource::Inline(_) => false,
            };

            let mut description = old
                .description
                .as_ref()
                .unwrap_or(&"".to_string())
                .to_owned();
            if has_fail_annotation {
                if !description.is_empty() {
                    description += "\n";
                }
                description += TEST_ANNOTATION_MSG;
            }
            if bad_graphql_skip {
                if !description.is_empty() {
                    description += "\n";
                }
                description += BAD_GRAPHQL_MSG;
            }

            let mut spec = format!("# {}\n", old.name);
            if !description.is_empty() {
                spec += &format!("\n{}\n", description);
            }

            if bad_graphql_skip {
                spec += "\n##### skip\n";
            } else if let Some(runner) = &old.runner {
                if *runner != Annotation::Fail {
                    spec += &format!(
                        "\n##### {}\n",
                        match runner {
                            Annotation::Only => "only",
                            Annotation::Skip => "skip",
                            Annotation::Fail => unreachable!(),
                        }
                    )
                } else {
                    println!("Automatically converting fail annotation in {:#?}. This builds the test suite, so this might take a while.", path);
                }
            }

            spec += "\n#### server:\n\n```";
            spec += &match &old.config {
                ConfigSource::File(path) => {
                    let path = PathBuf::from(path);

                    let ext = path.extension().unwrap().to_string_lossy().to_string();
                    let content = fs::read_to_string(path).expect("Failed to read config file");

                    format!(
                        "{}\n{}{}```\n\n",
                        ext,
                        content,
                        if content.ends_with('\n') { "" } else { "\n" },
                    )
                }
                ConfigSource::Inline(content) => {
                    format!(
                        "json\n{}\n```\n\n",
                        serde_json::to_string_pretty(&content).expect("Failed to serialize Config")
                    )
                }
            };

            if !old.mock.is_empty() {
                spec += &format!(
                    "#### mock:\n\n```yml\n{}\n```\n\n",
                    serde_yaml::to_string(&old.mock).expect("Failed to serialize mocks")
                );
            }

            if !old.env.is_empty() {
                spec += &format!(
                    "#### env:\n\n```yml\n{}\n```\n\n",
                    serde_yaml::to_string(&old.env).expect("Failed to serialize mocks")
                );
            }

            spec += &format!(
                "#### assert:\n\n```yml\n{}```\n",
                serde_yaml::to_string(
                    &old.assert
                        .iter()
                        .map(|x| x.request.0.clone())
                        .collect::<Vec<APIRequest>>()
                )
                .expect("Failed to serialize AssertSpec")
            );

            let md_path = PathBuf::from(format!("{}.md", file_stem));

            let mut f = File::options()
                .create(true)
                .write(true)
                .truncate(true)
                .open(execution_dir.join(&md_path))
                .expect("Failed to open execution spec");

            f.write_all(spec.as_bytes())
                .expect("Failed to write execution spec");

            if !has_fail_annotation {
                for (i, assert) in old.assert.iter().enumerate() {
                    let mut f = File::options()
                        .create(true)
                        .write(true)
                        .truncate(true)
                        .open(snapshots_dir.join(PathBuf::from(format!(
                            "execution_spec__{}.md_assert_{}.snap",
                            file_stem, i
                        ))))
                        .expect("Failed to open execution snapshot");

                    let mut res = assert.response.to_owned();

                    res.0.headers = res
                        .0
                        .headers
                        .into_iter()
                        .map(|(k, v)| (k.to_lowercase(), v.to_owned()))
                        .collect();

                    if !res.0.headers.contains_key("content-type") {
                        res.0
                            .headers
                            .insert("content-type".to_string(), "application/json".to_string());
                    }

                    res.0
                        .headers
                        .sort_by(|a, _, b, _| a.partial_cmp(b).unwrap());

                    let snap = format!(
                        "---\nsource: tests/execution_spec.rs\nexpression: response\n---\n{}\n",
                        serde_json::to_string_pretty(&res)
                            .expect("Failed to serialize assert.response"),
                    );

                    f.write_all(snap.as_bytes())
                        .expect("Failed to write exception spec");
                }
            }

            if !bad_graphql_skip {
                match &old.config {
                    ConfigSource::File(path) => {
                        let path = PathBuf::from(path);
                        let sdl = fs::read_to_string(path).expect("Failed to read config file");
                        generate_client_snapshot_sdl(&file_stem, &sdl, &reader).await;
                        generate_merged_snapshot_sdl(&file_stem, &sdl).await;
                    }
                    ConfigSource::Inline(config) => {
                        let config = reader
                            .resolve(config.to_owned())
                            .await
                            .expect("Failed to resolve config");
                        generate_client_snapshot(&file_stem, &config).await;
                        generate_merged_snapshot(&file_stem, &config).await;
                    }
                };
            }

            files_already_processed.insert(file_stem);
        } else if path.is_file() {
            println!("skipping unexpected file: {:?}", path);
        }
    }

    for x in read_dir(merge_dir).expect("Could not read graphql/merge directory") {
        let x = x.unwrap();

        let path = x.path();
        let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();

        if files_already_processed.contains(&file_stem) {
            panic!("File name collision: {}", file_stem);
        }

        if is_path_file_ext(&path, "graphql") {
            let spec = "\n".to_string()
                + read_to_string(&path)
                    .expect("Failed to read graphql/merge spec")
                    .as_str();

            let mut md_spec = format!("# {}\n\n", file_stem);

            let mut server: Vec<String> = Vec::with_capacity(2);
            let mut merged: Option<String> = None;

            for (typ, content) in graphql_iter_spec_part(&spec) {
                match typ.as_str() {
                    "server-sdl" => {
                        md_spec += &format!("#### server:\n\n```graphql\n{}\n```\n\n", content);
                        server.push(content);
                    }
                    "merged-sdl" => {
                        if merged.is_none() {
                            merged = Some(content);
                        } else {
                            panic!(
                                "Unexpected number of merged SDL declarations in {:?} (only one is allowed)",
                                path
                            );
                        }
                    }
                    _ => panic!("Unsupported part type in {:?}: {}", path, typ),
                };
            }

            if server.is_empty() {
                panic!("Unexpected number of server SDL declarations in {:?} (at least one is required, two are recommended)", path);
            }

            if merged.is_none() {
                panic!("Unexpected lack of merged SDL declarations in {:?}", path);
            }

            let md_path = PathBuf::from(format!("{}.md", file_stem));

            let mut f = File::options()
                .create(true)
                .write(true)
                .truncate(true)
                .open(execution_dir.join(&md_path))
                .expect("Failed to open execution spec");

            f.write_all(md_spec.as_bytes())
                .expect("Failed to write execution spec");

            let target = snapshots_dir.join(PathBuf::from(format!(
                "execution_spec__{}.md_merged.snap",
                file_stem,
            )));

            let snap = format!(
                "---\nsource: tests/execution_spec.rs\nexpression: merged\n---\n{}\n",
                merged.unwrap()
            );

            write(target, snap).expect("Failed to write merged snapshot");

            if server.len() == 1 {
                generate_client_snapshot_sdl(&file_stem, &server[0], &reader).await;
            }

            files_already_processed.insert(file_stem);
        } else if path.is_file() {
            println!("Skipping unexpected file: {:?}", path);
        }
    }

    for x in read_dir(client_dir).expect("Could not read graphql directory") {
        let x = x.unwrap();

        let path = x.path();
        let file_stem = path.file_stem().unwrap().to_string_lossy().to_string();

        if files_already_processed.contains(&file_stem) {
            panic!("File name collision: {}", file_stem);
        }

        if is_path_file_ext(&path, "graphql") {
            let spec = "\n".to_string()
                + read_to_string(&path)
                    .expect("Failed to read graphql spec")
                    .as_str();

            let mut server: Option<String> = None;
            let mut client: Option<String> = None;

            for (typ, content) in graphql_iter_spec_part(&spec) {
                match typ.as_str() {
                    "server-sdl" => {
                        if server.is_none() {
                            server = Some(content);
                        } else {
                            panic!(
                                "Unexpected number of server SDL declarations in {:?} (only one is allowed)",
                                path
                            );
                        }
                    }
                    "client-sdl" => {
                        if client.is_none() {
                            client = Some(content);
                        } else {
                            panic!(
                                "Unexpected number of client SDL declarations in {:?} (only one is allowed)",
                                path
                            );
                        }
                    }
                    _ => panic!("Unsupported part type in {:?}: {}", path, typ),
                };
            }

            if server.is_none() {
                panic!("Unexpected number of server SDL declarations in {:?} (at least one is required, two are recommended)", path);
            }

            let server = server.unwrap();

            let md_spec = format!(
                "# {}\n\n###### check identity\n\n#### server:\n\n```graphql\n{}\n```\n",
                file_stem, server,
            );

            if client.is_none() {
                panic!("Unexpected lack of client SDL declarations in {:?}", path);
            }

            let md_path = PathBuf::from(format!("{}.md", file_stem));

            let mut f = File::options()
                .create(true)
                .write(true)
                .truncate(true)
                .open(execution_dir.join(&md_path))
                .expect("Failed to open execution spec");

            f.write_all(md_spec.as_bytes())
                .expect("Failed to write execution spec");

            let target = snapshots_dir.join(PathBuf::from(format!(
                "execution_spec__{}.md_client.snap",
                file_stem,
            )));

            let snap = format!(
                "---\nsource: tests/execution_spec.rs\nexpression: merged\n---\n{}\n",
                client.unwrap()
            );

            write(target, snap).expect("Failed to write client snapshot");

            generate_merged_snapshot_sdl(&file_stem, &server).await;

            files_already_processed.insert(file_stem);
        } else if path.is_file() {
            println!("Skipping unexpected file: {:?}", path);
        }
    }

    for x in read_dir(errors_dir).expect("Could not read graphql/errors directory") {
        let x = x.unwrap();

        let path = x.path();
        let mut file_stem = path.file_stem().unwrap().to_string_lossy().to_string();

        if files_already_processed.contains(&file_stem) {
            println!(
                "File name collision: {}. Adding -error to the end.",
                file_stem
            );
            file_stem += "-error";
            if files_already_processed.contains(&file_stem) {
                panic!("File name collision: {}", file_stem);
            }
        }

        if is_path_file_ext(&path, "graphql") {
            let spec = "\n".to_string()
                + read_to_string(&path)
                    .expect("Failed to read graphql/errors spec")
                    .as_str();

            let mut server: Option<String> = None;
            let mut errors: Vec<SDLError> = Vec::new();

            for (typ, content) in graphql_iter_spec_part(&spec) {
                match typ.as_str() {
                    "server-sdl" => {
                        if server.is_none() {
                            server = Some(content);
                        } else {
                            panic!(
                                "Unexpected number of server SDL declarations in {:?} (only one is allowed)",
                                path
                            );
                        }
                    }
                    "client-sdl" => {
                        if content.contains("@error") {
                            let doc =
                                async_graphql::parser::parse_schema(content.as_str()).unwrap();
                            for def in doc.definitions {
                                if let TypeSystemDefinition::Type(type_def) = def {
                                    for dir in type_def.node.directives {
                                        if dir.node.name.node == "error" {
                                            errors.push(
                                                SDLError::from_directive(&dir.node)
                                                    .to_result()
                                                    .unwrap(),
                                            );
                                        }
                                    }
                                }
                            }
                        } else {
                            panic!("Unexpected lack of @error directives in {:?}", path);
                        }
                    }
                    _ => panic!("Unsupported part type in {:?}: {}", path, typ),
                };
            }

            if server.is_none() {
                panic!("Unexpected number of server SDL declarations in {:?} (exactly one is required)", path);
            }

            let md_spec = format!(
                "# {}\n\n###### sdl error\n\n#### server:\n\n```graphql\n{}\n```\n",
                file_stem,
                server.unwrap()
            );

            if errors.is_empty() {
                panic!("Unexpected lack of client SDL declarations in {:?}", path);
            }

            let md_path = PathBuf::from(format!("{}.md", file_stem));

            let mut f = File::options()
                .create(true)
                .write(true)
                .truncate(true)
                .open(execution_dir.join(&md_path))
                .expect("Failed to open execution spec");

            f.write_all(md_spec.as_bytes())
                .expect("Failed to write execution spec");

            let target = snapshots_dir.join(PathBuf::from(format!(
                "execution_spec__{}.md_errors.snap",
                file_stem,
            )));

            let snap = format!(
                "---\nsource: tests/execution_spec.rs\nexpression: errors\n---\n{}\n",
                serde_json::to_string_pretty(&errors).unwrap(),
            );

            write(target, snap).expect("Failed to write errors snapshot");

            files_already_processed.insert(file_stem);
        } else if path.is_file() {
            println!("Skipping unexpected file: {:?}", path);
        }
    }

    println!("Running prettier...");

    let prettierrc = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.prettierrc");
    let prettier = std::process::Command::new("prettier")
        .args([
            "-c",
            prettierrc.to_string_lossy().as_ref(),
            "--write",
            "tests/execution/*.md",
        ])
        .output()
        .expect("Failed to run prettier");

    if !prettier.status.success() {
        panic!(
            "prettier exited with an error:\n{}",
            String::from_utf8_lossy(&prettier.stdout)
        );
    }

    println!("All done!");
}
