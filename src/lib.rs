

mod core {
    pub use tailcall_core::core::*;
}

#[cfg(feature = "cli")]
pub mod cli;

// FIXME: make projects use tailcall-core instead of tailcall-core-core
pub use tailcall_core::*;
