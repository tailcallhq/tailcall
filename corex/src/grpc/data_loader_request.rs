use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

use anyhow::Result;

use super::request_template::RenderedRequestTemplate;

#[derive(Debug, Clone, Eq)]
pub struct DataLoaderRequest {
    pub template: RenderedRequestTemplate,
    batch_headers: BTreeSet<String>,
}

impl Hash for DataLoaderRequest {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.template.url.hash(state);
        self.template.body.hash(state);

        for name in &self.batch_headers {
            if let Some(value) = self.template.headers.get(name) {
                name.hash(state);
                value.hash(state);
            }
        }
    }
}

impl PartialEq for DataLoaderRequest {
    fn eq(&self, other: &Self) -> bool {
        let mut hasher_self = DefaultHasher::new();
        self.hash(&mut hasher_self);
        let hash_self = hasher_self.finish();

        let mut hasher_other = DefaultHasher::new();
        other.hash(&mut hasher_other);
        let hash_other = hasher_other.finish();

        hash_self == hash_other
    }
}

impl DataLoaderRequest {
    pub fn new(template: RenderedRequestTemplate, batch_headers: BTreeSet<String>) -> Self {
        Self { template, batch_headers }
    }

    pub fn to_request(&self) -> Result<reqwest::Request> {
        self.template.to_request()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    use hyper::header::{HeaderName, HeaderValue};
    use hyper::HeaderMap;
    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;
    use url::Url;

    use super::DataLoaderRequest;
    use crate::grpc::protobuf::{ProtobufOperation, ProtobufSet};
    use crate::grpc::request_template::RenderedRequestTemplate;

    static PROTOBUF_OPERATION: Lazy<ProtobufOperation> = Lazy::new(|| {
        let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        root_dir.pop();
        let mut test_file = root_dir.join(file!());

        test_file.pop();
        test_file.push("tests");
        test_file.push("greetings.proto");

        let protobuf_set = ProtobufSet::from_proto_file(&test_file).unwrap();
        let service = protobuf_set.find_service("Greeter").unwrap();

        service.find_operation("SayHello").unwrap()
    });

    #[test]
    fn dataloader_req_empty_headers() {
        let batch_headers = BTreeSet::default();
        let tmpl = RenderedRequestTemplate {
            url: Url::parse("http://localhost:3000/").unwrap(),
            headers: HeaderMap::new(),
            operation: PROTOBUF_OPERATION.clone(),
            body: "{}".to_owned(),
        };

        let dl_req_1 = DataLoaderRequest::new(tmpl.clone(), batch_headers.clone());
        let dl_req_2 = DataLoaderRequest::new(tmpl.clone(), batch_headers);

        assert_eq!(dl_req_1, dl_req_2);
    }

    #[test]
    fn dataloader_req_batch_headers() {
        let batch_headers = BTreeSet::from_iter(["test-header".to_owned()]);
        let tmpl_1 = RenderedRequestTemplate {
            url: Url::parse("http://localhost:3000/").unwrap(),
            headers: HeaderMap::from_iter([(
                HeaderName::from_static("test-header"),
                HeaderValue::from_static("value1"),
            )]),
            operation: PROTOBUF_OPERATION.clone(),
            body: "{}".to_owned(),
        };
        let tmpl_2 = tmpl_1.clone();

        let dl_req_1 = DataLoaderRequest::new(tmpl_1.clone(), batch_headers.clone());
        let dl_req_2 = DataLoaderRequest::new(tmpl_2, batch_headers.clone());

        assert_eq!(dl_req_1, dl_req_2);

        let tmpl_2 = RenderedRequestTemplate {
            headers: HeaderMap::from_iter([(
                HeaderName::from_static("test-header"),
                HeaderValue::from_static("value2"),
            )]),
            ..tmpl_1.clone()
        };
        let dl_req_2 = DataLoaderRequest::new(tmpl_2, batch_headers.clone());

        assert_ne!(dl_req_1, dl_req_2);
    }
}
