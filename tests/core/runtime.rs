extern crate core;

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use derive_setters::Setters;
use tailcall::blueprint;
use tailcall::cache::InMemoryCache;
use tailcall::cli::javascript;
use tailcall::config::Source;
use tailcall::runtime::TargetRuntime;

use super::env::TestEnvIO;
use super::file::TestFileIO;
use super::http::MockHttpClient;
use super::model::*;

#[derive(Clone, Setters)]
pub struct ExecutionSpec {
    pub path: PathBuf,
    pub name: String,
    pub safe_name: String,

    pub server: Vec<(Source, String)>,
    pub mock: Option<Vec<Mock>>,
    pub env: Option<HashMap<String, String>>,
    pub test: Option<Vec<APIRequest>>,
    pub files: BTreeMap<String, String>,

    // Annotations for the runner
    pub runner: Option<Annotation>,

    pub check_identity: bool,
    pub sdl_error: bool,
}

#[derive(Clone, Debug)]
pub struct ExecutionMock {
    pub mock: Mock,
    pub actual_hits: Arc<AtomicUsize>,
}

impl ExecutionMock {
    pub fn test_hits(&self, path: impl AsRef<Path>) {
        let url = &self.mock.request.0.url;
        let is_batch_graphql = url.path().starts_with("/graphql")
            && self
                .mock
                .request
                .0
                .body
                .as_str()
                .map(|s| s.contains(','))
                .unwrap_or_default();

        // do not test hits for mocks for batch graphql requests
        // since that requires having 2 mocks with different order of queries in
        // single request and only one of that mocks is actually called during run.
        // for other protocols there is no issues right now, because:
        // - for http the keys are always sorted https://github.com/tailcallhq/tailcall/blob/51d8b7aff838f0f4c362d4ee9e39492ae1f51fdb/src/http/data_loader.rs#L71
        // - for grpc body is not used for matching the mock and grpc will use grouping based on id https://github.com/tailcallhq/tailcall/blob/733b641c41f17c60b15b36b025b4db99d0f9cdcd/tests/execution_spec.rs#L769
        if is_batch_graphql {
            return;
        }

        let expected_hits = self.mock.expected_hits;
        let actual_hits = self.actual_hits.load(Ordering::Relaxed);

        assert_eq!(
            expected_hits,
            actual_hits,
            "expected mock for {url} to be hit exactly {expected_hits} times, but it was hit {actual_hits} times for file: {:?}",
            path.as_ref()
        );
    }
}

pub fn create_runtime(
    http_client: Arc<MockHttpClient>,
    env: Option<HashMap<String, String>>,
    script: Option<blueprint::Script>,
) -> TargetRuntime {
    let http = if let Some(script) = script.clone() {
        javascript::init_http(http_client.clone(), script)
    } else {
        http_client.clone()
    };

    let http2 = if let Some(script) = script {
        javascript::init_http(http_client.clone(), script)
    } else {
        http_client.clone()
    };

    let file = TestFileIO::init();
    let env = TestEnvIO::init(env);

    TargetRuntime {
        http,
        http2_only: http2,
        env: Arc::new(env),
        file: Arc::new(file),
        cache: Arc::new(InMemoryCache::new()),
        extensions: Arc::new(vec![]),
    }
}
