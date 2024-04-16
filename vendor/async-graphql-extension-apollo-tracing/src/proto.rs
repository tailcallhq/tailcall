#![allow(rustdoc::all)]
#![allow(clippy::all)]

#[allow(unknown_lints)]
#[allow(unused_attributes)]
#[cfg_attr(rustfmt, rustfmt::skip)]
#[allow(box_pointers)]
#[allow(dead_code)]
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(non_upper_case_globals)]
#[allow(trivial_casts)]
#[allow(unused_results)]
#[allow(unused_mut)]
pub mod reports {
    include!(concat!(env!("OUT_DIR"), concat!("/proto/reports.rs")));
}
