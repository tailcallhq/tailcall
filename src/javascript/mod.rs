#[cfg(feature = "js")]
mod enable_js;
#[cfg(feature = "js")]
pub use enable_js::*;

#[cfg(not(feature = "js"))]
mod runtime_no_js;
#[cfg(not(feature = "js"))]
pub use runtime_no_js::*;
