use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use anyhow::{Context, Result};
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};
use protox::file::{FileResolver, GoogleFileResolver};
use url::Url;

use crate::config::{Config, ExprBody};
use crate::{FileIO, HttpIO};

const NULL_STR: &str = "\0\0\0\0\0\0\0";

pub struct ProtoPathResolver {
    descriptors: HashMap<String, FileDescriptorProto>,
    file_io: Arc<dyn FileIO>,
    http_io: Arc<dyn HttpIO>,
}

impl ProtoPathResolver {
    pub fn init(file_io: Arc<dyn FileIO>, http_io: Arc<dyn HttpIO>) -> Self {
        Self { descriptors: HashMap::new(), file_io, http_io }
    }

    pub async fn get_descriptor_set(mut self, config: &Config) -> Result<FileDescriptorSet> {
        let mut set = FileDescriptorSet::default();
        for (_, typ) in config.types.iter() {
            for (_, fld) in typ.fields.iter() {
                let proto_path = if let Some(grpc) = &fld.grpc {
                    &grpc.proto_path
                } else if let Some(ExprBody::Grpc(grpc)) = fld.expr.as_ref().map(|e| &e.body) {
                    &grpc.proto_path
                } else {
                    NULL_STR
                };

                if proto_path != NULL_STR {
                    self.import_all(proto_path.to_string()).await?;
                }
            }
        }
        for (_, v) in self.descriptors {
            set.file.push(v);
        }
        Ok(set)
    }
    async fn import_all(&mut self, proto_path: String) -> Result<()> {
        let source = self.resolve(&proto_path).await?;

        let mut queue = VecDeque::new();
        let parent_proto = protox_parse::parse(&proto_path, &source)?;
        queue.push_back(parent_proto.clone());

        while let Some(file) = queue.pop_front() {
            for import in file.dependency.iter() {
                let source = self.resolve(import).await?;
                if self.descriptors.get(import).is_some() {
                    continue;
                }
                let fdp = protox_parse::parse(import, &source)?;
                queue.push_back(fdp.clone());
                self.descriptors.insert(import.clone(), fdp);
            }
        }

        self.descriptors.insert(proto_path, parent_proto);

        Ok(())
    }
    async fn resolve(&self, path: &str) -> Result<String> {
        if let Ok(file) = GoogleFileResolver::new().open_file(path) {
            return Ok(file
                .source()
                .context("Unable to extract content of google well-known proto file")?
                .to_string());
        }

        let source = match Url::parse(path) {
            Ok(url) => {
                let resp = self
                    .http_io
                    .execute(reqwest::Request::new(reqwest::Method::GET, url))
                    .await?
                    .body
                    .to_vec();
                String::from_utf8(resp.to_vec())?
            }
            Err(_) => self.file_io.read(path).await?,
        };
        Ok(source)
    }
}
