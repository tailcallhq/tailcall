use std::collections::HashSet;
use std::fs::{self, canonicalize, read_dir, read_to_string, File};
use std::io::Write;
use std::path::PathBuf;

use http::ConfigSource;
use tailcall::cli::{init_file, init_http};
use tailcall::config::reader::ConfigReader;
use tailcall::config::Upstream;

use crate::common::Annotation;

mod common;
mod execution;
mod http;

const TEST_ANNOTATION_MSG: &str = "**This test had an assertion with a fail annotation that testconv cannot convert losslessly.** If you need the original responses, you can find it in git history. (For example, at commit [1c32ca9](https://github.com/tailcallhq/tailcall/tree/1c32ca9e8080ae3b17e9cf41078d028d3e0289da))";
const BAD_GRAPHQL_MSG: &str = "This test has invalid GraphQL that wasn't caught by http_spec before conversion. It is skipped right now, but it should be fixed at some point.";

impl From<http::DownstreamAssertion> for execution::DownstreamAssertion {
    fn from(value: http::DownstreamAssertion) -> Self {
        Self { request: value.request.clone() }
    }
}

impl From<http::HttpSpec> for execution::AssertSpec {
    fn from(value: http::HttpSpec) -> Self {
        Self {
            mock: value.mock.clone(),
            assert: value.assert.clone().into_iter().map(|x| x.into()).collect(),
            env: value.env.clone(),
        }
    }
}

#[tokio::main]
async fn main() {
    let http_dir =
        canonicalize(PathBuf::from("tests/http")).expect("Could not find http directory");

    let merge_dir = canonicalize(PathBuf::from("tests/graphql/merge"))
        .expect("Could not find graphql/merge directory");

    let execution_dir =
        canonicalize(PathBuf::from("tests/execution")).expect("Could not find execution directory");

    let snapshots_dir =
        canonicalize(PathBuf::from("tests/snapshots")).expect("Could not find snapshots directory");

    let mut files_already_processed: HashSet<String> = HashSet::new();

    let reader = ConfigReader::init(init_file(), init_http(&Upstream::default(), None));

    for x in read_dir(http_dir).expect("Could not read http directory") {
        let x = x.unwrap();

        let path = x.path();
        let file_stem = path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_owned()
            .to_string();

        if files_already_processed.contains(&file_stem) {
            panic!("File name collision: {}", file_stem);
        }

        if path.is_file()
            && path
                .extension()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .as_str()
                == "yml"
        {
            let f = File::open(&path).expect("Failed to open http spec");

            let old = serde_yaml::from_reader::<File, http::HttpSpec>(f).unwrap();

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
                if runner.to_owned() != Annotation::Fail {
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

            spec += &format!("\n#### server:\n\n```");
            spec += &match &old.config {
                http::ConfigSource::File(path) => {
                    let path = PathBuf::from(path);

                    let ext = path.extension().unwrap().to_string_lossy().to_string();
                    let content = fs::read_to_string(path).expect("Failed to read config file");

                    format!(
                        "{}\n{}{}```\n\n",
                        ext,
                        content,
                        if content.ends_with("\n") { "" } else { "\n" },
                    )
                }
                http::ConfigSource::Inline(content) => {
                    format!(
                        "json\n{}\n```\n\n",
                        serde_json::to_string_pretty(&content).expect("Failed to serialize Config")
                    )
                }
            };

            spec += &format!(
                "#### assert:\n\n```yml\n{}```\n",
                serde_yaml::to_string(&execution::AssertSpec::from(old.clone()))
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

            if has_fail_annotation {
                let test = std::process::Command::new("cargo")
                    .env("INSTA_FORCE_PASS", "1")
                    .args([
                        "test",
                        "--no-fail-fast",
                        "-p",
                        "tailcall",
                        "--test",
                        "execution_spec",
                        "--",
                        execution_dir.join(&md_path).to_str().unwrap(),
                    ])
                    .output()
                    .expect("Failed to run cargo test");

                if !test.status.success() {
                    panic!(
                        "Running cargo test (needed for fail annotation conversion) failed:\n{}",
                        String::from_utf8_lossy(&test.stderr)
                    );
                }

                let mut patched = 0;

                for i in 0..old.assert.len() {
                    let old = snapshots_dir.join(PathBuf::from(format!(
                        "execution_spec__{}.md_assert_{}.snap",
                        file_stem, i
                    )));

                    let new = snapshots_dir.join(PathBuf::from(format!(
                        "execution_spec__{}.md_assert_{}.snap.new",
                        file_stem, i
                    )));

                    if new.exists() {
                        std::fs::rename(new, &old).unwrap();

                        let snap = fs::read_to_string(&old)
                            .expect("Failed to read back snapshot for patching");

                        let lines = snap
                            .split("\n")
                            .filter(|x| !x.starts_with("assertion_line"))
                            .collect::<Vec<&str>>()
                            .join("\n");

                        std::fs::write(&old, lines).expect("Failed to write back patched snapshot");

                        patched += 1;
                    }
                }

                if patched == 0 {
                    panic!(
                        "Spec {:?} has a fail annotation but all tests passed.",
                        path
                    );
                }
            }

            files_already_processed.insert(file_stem);
        } else if path.is_file() {
            println!("skipping unexpected file: {:?}", path);
        }
    }

    for x in read_dir(merge_dir).expect("Could not read graphql/merge directory") {
        let x = x.unwrap();

        let path = x.path();
        let file_stem = path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_owned()
            .to_string();

        if files_already_processed.contains(&file_stem) {
            panic!("File name collision: {}", file_stem);
        }

        if path.is_file()
            && path
                .extension()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .as_str()
                == "graphql"
        {
            let spec = "\n".to_string()
                + read_to_string(&path)
                    .expect("Failed to read graphql/merge spec")
                    .as_str();

            let mut md_spec = format!("# {}\n\n", path.file_name().unwrap().to_string_lossy());

            let mut server_cnt = 0;
            let mut merged_cnt = 0;

            for part in spec.split("\n#> ").skip(1) {
                let (typ, content) = part.split_once("\n").unwrap();

                // CRLF support
                let typ = typ.trim();
                let content = content.trim().to_string();

                match typ {
                    "server-sdl" => {
                        md_spec += &format!("#### server:\n\n```graphql\n{}\n```\n\n", content);
                        server_cnt += 1;
                    }
                    "merged-sdl" => {
                        md_spec += &format!("#### merged:\n\n```graphql\n{}\n```\n\n", content);
                        merged_cnt += 1;
                    }
                    _ => panic!("Unsupported part type in {:?}: {}", path, typ),
                };
            }

            if server_cnt < 1 {
                panic!("Unexpected number of server SDL declarations in {:?} (at least one is required, two are recommended)", path);
            }

            if merged_cnt != 1 {
                panic!(
                    "Unexpected number of merged SDL declarations in {:?} (only one is allowed)",
                    path
                );
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

            files_already_processed.insert(file_stem);
        } else if path.is_file() {
            println!("Skipping unexpected file: {:?}", path);
        }
    }

    println!("Running prettier...");

    let prettier = std::process::Command::new("prettier")
        .args(["-c", ".prettierrc", "--write", "tests/execution/*.md"])
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
