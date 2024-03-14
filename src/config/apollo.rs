use serde::{Deserialize, Serialize};

use crate::config::ConfigReaderContext;
use crate::mustache::Mustache;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Apollo {
    ///
    /// Setting `api_key` for Apollo.
    pub api_key: String,
    ///
    /// Setting `graph_ref` for Apollo in the format <graph_id>@<variant>.
    pub graph_ref: String,
    ///
    /// Setting `userVersion` for Apollo.
    #[serde(default = "default_user_version")]
    pub user_version: String,
    ///
    /// Setting `platform` for Apollo.
    #[serde(default = "default_platform")]
    pub platform: String,
    ///
    /// Setting `version` for Apollo.
    #[serde(default = "default_version")]
    pub version: String,
}

fn default_user_version() -> String {
    "1.0".to_string()
}

fn default_platform() -> String {
    "platform".to_string()
}

fn default_version() -> String {
    "1.0".to_string()
}

impl Apollo {
    pub fn render_mustache(&mut self, reader_ctx: &ConfigReaderContext) -> anyhow::Result<()> {
        let Apollo { api_key, graph_ref, user_version, platform, version } = self;

        let api_key_tmpl = Mustache::parse(api_key)?;
        *api_key = api_key_tmpl.render(reader_ctx);

        let graph_ref_tmpl = Mustache::parse(graph_ref)?;
        *graph_ref = graph_ref_tmpl.render(reader_ctx);

        let user_version_tmpl = Mustache::parse(user_version)?;
        *user_version = user_version_tmpl.render(reader_ctx);

        let platform_tmpl = Mustache::parse(platform)?;
        *platform = platform_tmpl.render(reader_ctx);

        let version_tmpl = Mustache::parse(version)?;
        *version = version_tmpl.render(reader_ctx);

        Ok(())
    }
}
