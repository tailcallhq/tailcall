use std::sync::Arc;

use crate::blueprint::Upstream;
use crate::cache::InMemoryCache;
use crate::cli::javascript;
use crate::target_runtime::TargetRuntime;
use crate::test_rt::env::TestEnvIO;
use crate::test_rt::file::TestFileIO;
use crate::test_rt::http::TestHttp;
use crate::{blueprint, HttpIO};

mod env;
mod file;
mod http;

pub fn init_test_rt(script: Option<blueprint::Script>) -> TargetRuntime {
    let http: Arc<dyn HttpIO + Sync + Send> = if let Some(script) = script.clone() {
        javascript::init_http(TestHttp::init(&Default::default()), script)
    } else {
        Arc::new(TestHttp::init(&Default::default()))
    };

    let http2: Arc<dyn HttpIO + Sync + Send> = if let Some(script) = script {
        javascript::init_http(
            TestHttp::init(&Upstream::default().http2_only(true)),
            script,
        )
    } else {
        Arc::new(TestHttp::init(&Upstream::default().http2_only(true)))
    };

    let file = TestFileIO::init();
    let env = TestEnvIO::init();

    TargetRuntime {
        http,
        http2_only: http2,
        env: Arc::new(env),
        file: Arc::new(file),
        cache: Arc::new(InMemoryCache::new()),
    }
}
