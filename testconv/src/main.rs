use std::collections::HashSet;
use std::fs::{self, canonicalize, read_dir, File};
use std::io::Write;
use std::path::PathBuf;

mod common;
mod execution;
mod http;

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

fn main() {
    let http_dir =
        canonicalize(PathBuf::from("tests/http")).expect("Could not find http directory");

    let execution_dir =
        canonicalize(PathBuf::from("tests/execution")).expect("Could not find execution directory");

    let snapshots_dir =
        canonicalize(PathBuf::from("tests/snapshots")).expect("Could not find snapshots directory");

    let mut files_already_processed: HashSet<String> = HashSet::new();

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

            let mut spec = format!("# {}\n", old.name);
            if let Some(description) = &old.description {
                spec += &format!("{}\n", description);
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

            let mut f = File::options()
                .create(true)
                .write(true)
                .truncate(true)
                .open(execution_dir.join(PathBuf::from(format!("{}.md", file_stem))))
                .expect("Failed to open execution spec");

            f.write_all(spec.as_bytes())
                .expect("Failed to open execution spec");

            for (i, assert) in old.assert.into_iter().enumerate() {
                let mut f = File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(snapshots_dir.join(PathBuf::from(format!(
                        "execution_spec__{}.md_assert_{}.snap",
                        file_stem, i
                    ))))
                    .expect("Failed to open execution snapshot");

                let mut res = assert.response;
                if !res.0.headers.contains_key("content-type") {
                    res.0
                        .headers
                        .insert("content-type".to_string(), "application/json".to_string());
                }

                let snap = format!(
                    "---\nsource: tests/execution_spec.rs\nexpression: response\n---\n{}",
                    serde_json::to_string_pretty(&res)
                        .expect("Failed to serialize assert.response"),
                );

                f.write_all(snap.as_bytes())
                    .expect("Failed to write exception spec");
            }

            files_already_processed.insert(file_stem);
        } else if path.is_file() {
            println!("skipping unexpected file: {:?}", path);
        }
    }

    println!("Hello, world!");
}
