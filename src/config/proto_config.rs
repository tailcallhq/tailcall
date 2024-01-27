use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use anyhow::Result;
use prost_reflect::prost_types::{FileDescriptorProto, FileDescriptorSet};

use crate::config::Config;
use crate::{FileIO, HttpIO, ProtoPathResolver};

#[allow(clippy::too_many_arguments)]
async fn import_all(
    map: &mut HashMap<String, FileDescriptorProto>,
    proto_path: String,
    file_io: Arc<dyn FileIO>,
    http_io: Arc<dyn HttpIO>,
    resolver: Arc<dyn ProtoPathResolver>,
) -> Result<()> {
    let source = resolver
        .resolve(&proto_path, http_io.clone(), file_io.clone())
        .await?;

    let mut queue = VecDeque::new();
    let parent_proto = protox_parse::parse(&proto_path, &source)?;
    queue.push_back(parent_proto.clone());

    while let Some(file) = queue.pop_front() {
        for import in file.dependency.iter() {
            let source = resolver
                .resolve(import, http_io.clone(), file_io.clone())
                .await?;
            if map.get(import).is_some() {
                continue;
            }
            let fdp = protox_parse::parse(import, &source)?;
            queue.push_back(fdp.clone());
            map.insert(import.clone(), fdp);
        }
    }

    map.insert(proto_path, parent_proto);

    Ok(())
}

pub async fn get_descriptor_set(
    config: &Config,
    file_io: Arc<dyn FileIO>,
    http_io: Arc<dyn HttpIO>,
    resolver: Arc<dyn ProtoPathResolver>,
) -> Result<FileDescriptorSet> {
    let mut set = FileDescriptorSet::default();
    let mut hashmap = HashMap::new();
    for (_, typ) in config.types.iter() {
        for (_, fld) in typ.fields.iter() {
            if let Some(grpc) = &fld.grpc {
                let proto_path = grpc.proto_path.clone();
                import_all(
                    &mut hashmap,
                    proto_path,
                    file_io.clone(),
                    http_io.clone(),
                    resolver.clone(),
                )
                .await?;
            }
        }
    }
    for (_, v) in hashmap {
        set.file.push(v);
    }
    Ok(set)
}
