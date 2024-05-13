extern crate core;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use derive_setters::Setters;
use tailcall::cli::javascript;
use tailcall::{InMemoryCache, Script, Source, TargetRuntime};
use tailcall_hasher::TailcallHashMap;

use super::env::Env;
use super::file::TestFileIO;
use super::http::Http;
use super::model::*;

#[derive(Clone, Setters)]
pub struct ExecutionSpec {
    pub path: PathBuf,
    pub name: String,
    pub safe_name: String,

    pub server: Vec<(Source, String)>,
    pub mock: Option<Vec<Mock>>,
    pub env: Option<TailcallHashMap<String, String>>,
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
        if !self.mock.assert_hits {
            return;
        }
        let expected_hits = self.mock.expected_hits;
        let actual_hits = self.actual_hits.load(Ordering::Relaxed);

        assert_eq!(
            expected_hits,
            actual_hits,
            "expected mock for {} to be hit exactly {} times, but it was hit {} times for file: {:?}", url, expected_hits, actual_hits,
            path.as_ref()
        );
    }
}

pub fn create_runtime(
    http_client: Arc<Http>,
    env: Option<TailcallHashMap<String, String>>,
    script: Option<Script>,
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
    let env = Env::init(env);

    TargetRuntime {
        http,
        http2_only: http2,
        env: Arc::new(env),
        file: Arc::new(file),
        cache: Arc::new(InMemoryCache::new()),
        extensions: Arc::new(vec![]),
    }
}
