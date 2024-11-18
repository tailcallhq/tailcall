use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default, schemars::JsonSchema,
)]
pub enum EncodingUtil {
    #[default]
    ApplicationJson,
    ApplicationXWwwFormUrlencoded,
}
