use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{bracketed, parse_macro_input, Ident, ItemFn, Token, TypeBareFn, TypePath};

/// internal encoding of registry name
/// declare_registry!(unwind)
/// #[kernel_test(unwind)]
/// `unwind` will be internally encoded as
/// __TESTER_($CRATE_NAME)_REGISTRY_UNWIND
/// a declared slice is only visible for modules within the same crate
/// $CRATE_NAME is retrieved in proc-macro with env!("CARGO_CRATE_NAME")
/// in declarative macro with $crate
fn encode_registry_name(registry_name: &Ident) -> Ident {
    // fetch crate name where the proc macro is expanded, need this to form a crate namespace boundary
    let crate_name = std::env::var("CARGO_PKG_NAME").unwrap().to_uppercase();
    format_ident!(
        "__TESTER_{}_REGISTRY_{}",
        crate_name,
        registry_name.to_string().to_uppercase(),
        span = registry_name.span()
    )
}

#[proc_macro_attribute]
pub fn kernel_test(arg: TokenStream, input: TokenStream) -> TokenStream {
    let registry_name = encode_registry_name(&parse_macro_input!(arg as Ident));
    let test_fn = parse_macro_input!(input as ItemFn);
    if test_fn.sig.inputs.iter().len() != 0 {
        return syn::Error::new(
            test_fn.sig.ident.span(),
            "fn to register as test unit should not have any arguments",
        )
        .to_compile_error()
        .into();
    }
    let test_fn_output = &test_fn.sig.output;
    let test_fn_ident = &test_fn.sig.ident;
    let test_fn_ident_str = test_fn_ident.to_string();
    let fn_ty = quote! {
        fn() #test_fn_output
    };
    let dummy_static_var = format_ident!(
        "{}_{}",
        registry_name,
        test_fn_ident_str,
        span = test_fn_ident.span(),
    );
    quote! {
        #[allow(non_upper_case_globals)]
        #[::linkme::distributed_slice(crate::#registry_name)]
        static #dummy_static_var: ::tester::TestUnit<#fn_ty> = ::tester::TestUnit {
            test_fn: #test_fn_ident,
            info: ::tester::TestFnInfo {
                file: core::file!(),
                fn_name: #test_fn_ident_str,
                line: core::line!() + 1,
                column: core::column!(),
            }
        };
        #test_fn
    }
    .into()
}

struct RegistryConfig {
    name: Ident,
    fn_ty: TypeBareFn,
}

impl Parse for RegistryConfig {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let fn_ty = if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            let content;
            bracketed!(content in input);
            content.parse::<TypeBareFn>()?
        } else {
            syn::parse_quote! {
                fn() -> bool
            }
        };
        Ok(Self { name, fn_ty })
    }
}

/// allow user to declare a crate level registry to which they can submit test units
/// declare_registry!(name: [fn()])
/// fn type of registry is default to `fn() -> bool` if omitted
#[proc_macro]
pub fn declare_registry(input: TokenStream) -> TokenStream {
    let registry_config = parse_macro_input!(input as RegistryConfig);
    let linkme_slice_ident = encode_registry_name(&registry_config.name);
    let registry_fn_ty = &registry_config.fn_ty;

    quote! {
        #[allow(non_upper_case_globals)]
        #[::linkme::distributed_slice]
        static #linkme_slice_ident: [::tester::TestUnit<#registry_fn_ty>];
    }
    .into()
}

/// run_all!($registry_name, $runner_name)
/// iterate and run all test units in a registry slice
/// user should provide a runner which executes each of the test unit in the registry
///
/// runner fn takes fn in the registry as input and outputs a boolean to indicate whether test unit is passed or not
/// runner can also be omitted if
///     1. registry contains functions of type fn() -> bool
///     2. the returned boolean indicates whether the test is passed or not
struct RunConfig {
    registry_name: Ident,
    runner: Option<TypePath>,
}

impl Parse for RunConfig {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let registry_name: Ident = input.parse()?;
        let runner = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            Some(input.parse::<TypePath>()?)
        } else {
            None
        };
        Ok(Self {
            registry_name,
            runner,
        })
    }
}

#[proc_macro]
pub fn test_all(input: TokenStream) -> TokenStream {
    let run_config = parse_macro_input!(input as RunConfig);
    let registry_slice_name = encode_registry_name(&run_config.registry_name);
    let runner = run_config.runner.map_or(
        quote! {
            |f: fn() -> bool| f()
        },
        |runner| quote! {#runner},
    );
    quote! {{
        let binded_struct = ::tester::TestRegistry::new(&#registry_slice_name, #runner);
        binded_struct.run_all()
    }}
    .into()
}
