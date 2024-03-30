use std::iter::Extend;

use anyhow::{Context, Result};
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use protox::file::{FileResolver, GoogleFileResolver};
use url::Url;

use crate::config::reader::ConfigReader;
use crate::config::Config;
use crate::config_generator::from_proto::from_proto;
use crate::config_generator::source::GeneratorSource;
use crate::runtime::TargetRuntime;

pub struct GeneratorReader {
    runtime: TargetRuntime,
    reader: ConfigReader, // TODO
}

struct FileRead {
    content: String,
    #[allow(dead_code)] // TODO drop this
    source: GeneratorSource,
    #[allow(dead_code)]
    path: String,
}

impl GeneratorReader {
    pub fn init(runtime: TargetRuntime) -> Self {
        Self {
            runtime: runtime.clone(),
            reader: ConfigReader::init(runtime),
        }
    }
    pub async fn read_all<T: AsRef<str>>(
        &self,
        files: &[T],
        input_ty: GeneratorSource,
        query: &str,
    ) -> Result<Config> {
        let mut descriptors = FileDescriptorSet::default();

        for file in files {
            match input_ty {
                GeneratorSource::PROTO => {
                    let parent_descriptor = self.read_proto(file).await?;
                    descriptors
                        .file
                        .extend(self.reader.resolve_descriptors(parent_descriptor).await?);
                }
            }
        }

        Ok(from_proto(&[descriptors], query))
    }

    /// Reads a file from the filesystem or from an HTTP URL
    async fn read_file<T: AsRef<str>>(&self, file: T) -> anyhow::Result<FileRead> {
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
    async fn read_proto<T: AsRef<str>>(&self, path: T) -> anyhow::Result<FileDescriptorProto> {
        let content = if let Ok(file) = GoogleFileResolver::new().open_file(path.as_ref()) {
            file.source()
                .context("Unable to extract content of google well-known proto file")?
                .to_string()
        } else {
            self.read_file(path.as_ref()).await?.content
        };

        Ok(protox_parse::parse(path.as_ref(), &content)?)
    }
}
