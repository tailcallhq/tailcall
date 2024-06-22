extern crate proc_macro;

use proc_macro::TokenStream;

mod merge_right;
mod positioned_config;

use crate::merge_right::expand_merge_right_derive;
use crate::positioned_config::expand_positioned_config;

#[proc_macro_derive(MergeRight, attributes(merge_right))]
pub fn merge_right_derive(input: TokenStream) -> TokenStream {
    expand_merge_right_derive(input)
}

#[proc_macro_derive(PositionedConfig, attributes(positioned_field))]
pub fn positioned_config(input: TokenStream) -> TokenStream {
    expand_positoned_config(input)
}
