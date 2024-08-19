use serde::{Deserialize, Serialize};

use crate::core::config::ConfigReaderContext;
use crate::core::is_default;
use crate::core::mustache::Mustache;

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
    #[serde(default, skip_serializing_if = "is_default")]
    pub user_version: Option<String>,
    ///
    /// Setting `platform` for Apollo.
    #[serde(default, skip_serializing_if = "is_default")]
    pub platform: Option<String>,
    ///
    /// Setting `version` for Apollo.
    #[serde(default, skip_serializing_if = "is_default")]
    pub version: Option<String>,
}

impl Apollo {
    pub fn render_mustache(&mut self, reader_ctx: &ConfigReaderContext) -> anyhow::Result<()> {
        let Apollo { api_key, graph_ref, user_version, platform, version } = self;

        let api_key_tmpl = Mustache::parse(api_key);
        *api_key = api_key_tmpl.render(reader_ctx);

        let graph_ref_tmpl = Mustache::parse(graph_ref);
        *graph_ref = graph_ref_tmpl.render(reader_ctx);

        let user_version_tmpl = Mustache::parse(user_version.as_deref().unwrap_or_default());
        *user_version = Some(user_version_tmpl.render(reader_ctx));

        let platform_tmpl = Mustache::parse(platform.as_deref().unwrap_or_default());
        *platform = Some(platform_tmpl.render(reader_ctx));

        let version_tmpl = Mustache::parse(version.as_deref().unwrap_or_default());
        *version = Some(version_tmpl.render(reader_ctx));

        Ok(())
    }
}
