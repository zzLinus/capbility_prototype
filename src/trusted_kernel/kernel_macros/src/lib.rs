use proc_macro::TokenStream;
mod unwind_point;

#[proc_macro_attribute]
pub fn trusted_kernel_export(args: TokenStream, input: TokenStream) -> TokenStream {
    unwind_point::trusted_kernel_export_impl(args, input)
}

#[proc_macro]
pub fn trusted_kernel_invoke(input: TokenStream) -> TokenStream {
    unwind_point::trusted_kernel_invoke_impl(input)
}
