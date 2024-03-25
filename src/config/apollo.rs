use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::config::ConfigReaderContext;
use crate::mustache::Mustache;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Apollo {
    ///
    /// Setting `apiKey` for Apollo.
    pub api_key: String,
    ///
    /// Setting `graphRef` for Apollo in the format <graphId>@<variant>.
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
        *api_key = api_key_tmpl
            .render_string(reader_ctx)
            .context("apiKey is not defined")?;

        let graph_ref_tmpl = Mustache::parse(graph_ref)?;
        *graph_ref = graph_ref_tmpl
            .render_string(reader_ctx)
            .context("graphRef is not defined")?;

        let user_version_tmpl = Mustache::parse(user_version)?;
        if let Some(rendered) = user_version_tmpl.render_string(reader_ctx) {
            *user_version = rendered;
        }

        let platform_tmpl = Mustache::parse(platform)?;
        if let Some(rendered) = platform_tmpl.render_string(reader_ctx) {
            *platform = rendered;
        }

        let version_tmpl = Mustache::parse(version)?;
        if let Some(rendered) = version_tmpl.render_string(reader_ctx) {
            *version = rendered;
        }

        Ok(())
    }
}
