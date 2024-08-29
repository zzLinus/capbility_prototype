#![no_std]
#![allow(unused)]
extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn export_interface(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let output = quote! {
        #[no_mangle]
        #[link_section=".export_code"]
        #input
    };

    TokenStream::from(output)
}
