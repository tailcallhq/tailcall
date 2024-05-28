use tailcall_version::VERSION;

const UTM_MEDIUM: &str = "server";
const DEBUG_UTM_SOURCE: &str = "tailcall-debug";
const RELEASE_UTM_SOURCE: &str = "tailcall-release";
const BASE_PLAYGROUND_URL: &str = "https://tailcall.run/playground/";

pub fn build_url(graphiql_url: &str) -> String {
    let utm_source = if VERSION.is_dev() {
        DEBUG_UTM_SOURCE
    } else {
        RELEASE_UTM_SOURCE
    };

    format!(
        "{}?u={}&utm_source={}&utm_medium={}",
        BASE_PLAYGROUND_URL, graphiql_url, utm_source, UTM_MEDIUM
    )
}
