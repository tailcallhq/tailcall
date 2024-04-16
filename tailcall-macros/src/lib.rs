extern crate proc_macro;

use proc_macro::TokenStream;

mod merge_right;

use crate::merge_right::expand_merge_right_derive;


#[proc_macro_derive(MergeRight, attributes(merge_right))]
pub fn merge_right_derive(input: TokenStream) -> TokenStream {
    expand_merge_right_derive(input)
}
