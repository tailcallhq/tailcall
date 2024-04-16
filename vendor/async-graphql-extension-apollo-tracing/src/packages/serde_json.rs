
#[cfg(not(target_arch = "wasm32"))]
pub use serde_json::*;
#[cfg(target_arch = "wasm32")]
pub use serde_json_wasm::*;
