use std::collections::{HashMap, VecDeque};
use std::iter::Extend;

use anyhow::{Context, Result};
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use protox::file::{FileResolver, GoogleFileResolver};
use url::Url;

use crate::config::Config;
use crate::config_generator::from_proto::from_proto;
use crate::config_generator::source::GeneratorSource;
use crate::runtime::TargetRuntime;

pub struct GeneratorReader {
    runtime: TargetRuntime,
}

struct FileRead {
    content: String,
    #[allow(dead_code)] // TODO drop this
    source: GeneratorSource,
    path: String,
}

impl GeneratorReader {
    pub fn init(runtime: TargetRuntime) -> Self {
        Self { runtime: runtime.clone() }
    }
    pub async fn read_all<T: AsRef<str>>(&self, files: &[T], query: &str) -> Result<Config> {
        let mut descriptors = FileDescriptorSet::default();

        for file in files {
            let file_read = self.read_file(file).await?;
            match file_read.source {
                GeneratorSource::PROTO => {
                    let parent_descriptor = self.read_proto(file_read).await?;
                    descriptors
                        .file
                        .extend(self.resolve_descriptors(parent_descriptor).await?);
                }
            }
        }

        Ok(from_proto(&[descriptors], query))
    }

    /// Reads a file from the filesystem or from an HTTP URL
    async fn read_file<T: AsRef<str>>(&self, file: T) -> Result<FileRead> {
        if let Ok(file) = GoogleFileResolver::new().open_file(file.as_ref()) {
            let content = file
                .source()
                .context("Unable to extract content of google well-known proto file")?
                .to_string();
            return Ok(FileRead {
                content,
                source: GeneratorSource::PROTO,
                path: file.name().to_string(),
            });
        }
        // Is an HTTP URL
        let file_read = if let Ok(url) = Url::parse(file.as_ref()) {
            if url.scheme().starts_with("http") {
                let response = self
                    .runtime
                    .http
                    .execute(reqwest::Request::new(reqwest::Method::GET, url))
                    .await?;
                let source = if let Some(content_ty) = response.headers.get("content-type") {
                    GeneratorSource::detect(content_ty.to_str()?)
                        .unwrap_or(GeneratorSource::detect(file.as_ref())?)
                } else {
                    GeneratorSource::detect(file.as_ref())?
                };

                FileRead {
                    content: String::from_utf8(response.body.to_vec())?,
                    source,
                    path: file.as_ref().to_string(),
                }
            } else {
                // Is a file path on Windows
                let source = GeneratorSource::detect(file.as_ref())?;
                let content = self.runtime.file.read(file.as_ref()).await?;
                FileRead { content, source, path: file.as_ref().to_string() }
            }
        } else {
            // Is a file path
            let source = GeneratorSource::detect(file.as_ref())?;
            let content = self.runtime.file.read(file.as_ref()).await?;
            FileRead { content, source, path: file.as_ref().to_string() }
        };

        Ok(file_read)
    }

    /// Tries to load well-known google proto files and if not found uses normal
    /// file and http IO to resolve them
    async fn read_proto(&self, file_read: FileRead) -> Result<FileDescriptorProto> {
        Ok(protox_parse::parse(
            file_read.path.as_ref(),
            &file_read.content,
        )?)
    }

    /// Performs BFS to import all nested proto files
    pub async fn resolve_descriptors(
        &self,
        parent_proto: FileDescriptorProto,
    ) -> Result<Vec<FileDescriptorProto>> {
        let mut descriptors: HashMap<String, FileDescriptorProto> = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back(parent_proto.clone());

        while let Some(file) = queue.pop_front() {
            for import in file.dependency.iter() {
                let file_read = self.read_file(import).await?;
                let proto = self.read_proto(file_read).await?;
                if descriptors.get(import).is_none() {
                    queue.push_back(proto.clone());
                    descriptors.insert(import.clone(), proto);
                }
            }
        }
        let mut descriptors = descriptors
            .into_values()
            .collect::<Vec<FileDescriptorProto>>();
        descriptors.push(parent_proto);
        Ok(descriptors)
    }
}
