use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Token,
};

/// caller should specify the name of linkme distributed slice to which the exported fn will be registerd
/// caller can optionally set `name` to configure the name of api visible to other crate, function name will be used as default
/// # Example
/// ```
/// use elsewhere::LINKME_SLICE
/// #[trusted_kernel_export(LINKME_SLICE)]
/// optionally
/// #[trusted_kernel_export(LINKME_SLICE, name = "api name")]
/// also allow direct export where distributed slice API_REGISTRY will be used
///
/// TODO: support generics
/// #[trusted_kernel_export(T = [usize, isize, String])]
/// fn template<T>()
/// otherwise, if no template specialization is provided
/// we do build.rs binding
/// ```
struct ExportConfig {
    global_slice_name: Option<syn::Path>,
    exported_name: Option<syn::Ident>,
}

impl ExportConfig {
    fn parse_key_val(input: ParseStream<'_>) -> syn::parse::Result<(syn::Ident, syn::Ident)> {
        let key: syn::Ident = input.parse()?;
        input.parse::<Token![=]>()?;
        assert!(key == "name");
        let val = if let syn::Lit::Str(ref lit) = input.parse::<syn::Lit>()? {
            syn::Ident::new(&lit.value(), proc_macro2::Span::call_site())
        } else {
            panic!("unsupported format to overwrite exposed api name, try #[trusted_kernel(name = \"<new name>\")]")
        };
        Ok((key, val))
    }
}

impl Parse for ExportConfig {
    fn parse(input: ParseStream<'_>) -> syn::parse::Result<Self> {
        if input.peek(syn::Ident) && input.peek2(Token![=]) && input.peek3(syn::Lit) {
            // parse pattern: #[trusted_kernel_export(name = "customed_name")]
            let key_val = Self::parse_key_val(input)?;
            Ok(Self {
                global_slice_name: None,
                exported_name: Some(key_val.1),
            })
        } else {
            let global_slice_name = if input.peek(syn::Ident) || input.peek2(Token![::]) {
                // parse pattern: #[trusted_kernel_export(SLICE_NAME)]
                Some(input.parse::<syn::Path>()?)
            } else {
                None
            };
            let exported_name = if input.parse::<Token![,]>().is_ok() {
                let key_val = Self::parse_key_val(input)?;
                Some(key_val.1)
            } else {
                None
            };
            Ok(Self {
                global_slice_name,
                exported_name,
            })
        }
    }
}

pub fn trusted_kernel_export_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let export_config = syn::parse_macro_input!(args as ExportConfig);

    let fn_to_export = syn::parse_macro_input!(input as syn::ItemFn);
    let fn_ident = fn_to_export.sig.ident.clone();
    let exported_name = export_config
        .exported_name
        .map_or_else(|| fn_ident.to_string(), |name| name.to_string());
    let global_slice_name = export_config.global_slice_name.map_or_else(
        || quote! {::trusted_kernel::API_REGISTRY},
        |slice_name| quote! {#slice_name},
    );

    let args_ty_list: Vec<_> = fn_to_export
        .sig
        .inputs
        .iter()
        .map(|arg| {
            if let syn::FnArg::Typed(syn::PatType { ty, .. }) = arg {
                ty.clone()
            } else {
                panic!("struct method is not supported to export, self like parameter is invalid");
            }
        })
        .collect();
    let args_index: Vec<_> = (0..args_ty_list.len())
        .map(|i| syn::LitInt::new(&format!("{}", i), proc_macro2::Span::call_site()))
        .collect();

    let ret_ty = if let syn::ReturnType::Type(_, ref ret_ty) = fn_to_export.sig.output {
        quote! {#ret_ty}
    } else {
        quote! {()}
    };

    let dummy_struct_ident = quote::format_ident!("__PrivateKernelExport_{}", fn_ident);
    let interface_proxy = quote::format_ident!("__private_kernel_export_{}_proxy", fn_ident);
    quote! {
        #[allow(non_camel_case_types)]
        struct #dummy_struct_ident;

        impl ::trusted_kernel::GlobalInterface for #dummy_struct_ident{
            fn path(&self) -> alloc::string::String {
                alloc::string::String::from(concat!(core::env!("CARGO_CRATE_NAME"), "::", #exported_name))
            }

            /// # Safety
            /// manipulation of raw pointer of args and ret_place_holder are self-contained
            /// args points to box allocated tuple of Option
            /// ret_place_holder options to box allocated Option<MaybeUninit<T>>
            /// MaybeUninit is guaranteed to be transparent, we treat it as Option<T>
            fn transmute_then_invoke(&self, args: *const (), ret_place_holder: *mut ()) {
                unsafe {
                    let mut args = alloc::boxed::Box::from_raw(args as *mut (#(Option<#args_ty_list>,)*));
                    let ret_place_holder = (&mut *(ret_place_holder as *mut Option<#ret_ty>)).as_mut().unwrap();
                    *ret_place_holder = #fn_ident(#(args.#args_index.take().unwrap()),*);
                }
                // args drop here
            }

            fn get_identifier(&self) -> ::trusted_kernel::ExportedAPIIdentifier {
                let args_hash = core::any::TypeId::of::<(#(#args_ty_list,)*)>();
                let ret_hash = core::any::TypeId::of::<#ret_ty>();
                ::trusted_kernel::ExportedAPIIdentifier::new(self.path(), args_hash, ret_hash)
            }
        }
        #fn_to_export


        // enforce Send and Sync for this dummy struct is a free be
        #[::linkme::distributed_slice(#global_slice_name)]
        fn #interface_proxy() -> alloc::boxed::Box<dyn ::trusted_kernel::GlobalInterface + Send + Sync>{
            alloc::boxed::Box::new(#dummy_struct_ident)
        }
    }
    .into()
}

struct InvokeVariadic {
    func_full_path: syn::Path,
    args_list: Vec<syn::PatType>,
    ret_ty: Option<syn::TypePath>,
}

impl Parse for InvokeVariadic {
    fn parse(input: ParseStream<'_>) -> syn::parse::Result<Self> {
        let func_full_path: syn::Path = input.parse()?;
        let mut args_list: Vec<syn::PatType> = vec![];
        let content;
        syn::parenthesized!(content in input);
        while !content.is_empty() {
            args_list.push(content.parse()?);
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        let ret_ty = if input.peek(Token![->]) {
            input.parse::<Token![->]>()?;
            Some(input.parse::<syn::TypePath>()?)
        } else {
            None
        };
        Ok(Self {
            func_full_path,
            args_list,
            ret_ty,
        })
    }
}

fn get_identifier(invoker: &InvokeVariadic) -> proc_macro2::TokenStream {
    // we might add another layer of transformation between usual mod path `crate_name::func_name::...` to trusted_kernel inner api name encoding
    // for now, those two are identical
    let api_name = invoker
        .func_full_path
        .segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    let args_hash = {
        let args_ty_list = invoker.args_list.iter().map(|arg| arg.ty.clone());
        quote! {
            core::any::TypeId::of::<(#(#args_ty_list,)*)>()
        }
    };
    let ret_ty = invoker
        .ret_ty
        .as_ref()
        .map(|ret_ty| quote! {#ret_ty})
        .unwrap_or(quote! {()});
    quote! {
        ::trusted_kernel::ExportedAPIIdentifier::new(
            #api_name, #args_hash, core::any::TypeId::of::<#ret_ty>()
        )
    }
}

/// entry point for trusted_kernel service to invoke api exposed from other service
/// this invoke guarantees type safety as compile time
/// # Example
/// ```
/// // service mm expose one api with protype fn callme(s: String, i: usize) -> isize with #[trusted_kernel_export]
/// // in other service crate
/// let s = String::from("hello");
/// let i = 100;
/// let ret = trusted_kernel_invoke!(
///     mm::callme(s: String, i: usize) -> isize
/// ).unwrap()
/// ```
pub fn trusted_kernel_invoke_impl(input: TokenStream) -> TokenStream {
    let func_invoke = syn::parse_macro_input!(input as InvokeVariadic);
    let api_identifier = get_identifier(&func_invoke);
    let args_val: Vec<_> = func_invoke
        .args_list
        .iter()
        .map(|arg_and_ty| {
            let arg_val = arg_and_ty.pat.clone();
            quote! {#arg_val}
        })
        .collect();
    let args_ty: Vec<_> = func_invoke
        .args_list
        .iter()
        .map(|arg_and_ty| {
            let ty = arg_and_ty.ty.clone();
            quote! {#ty}
        })
        .collect();
    let ret_ty = func_invoke
        .ret_ty
        .as_ref()
        .map(|ret_ty| quote! {#ret_ty})
        .unwrap_or(quote! {()});

    quote! {{
        let api_identifier = #api_identifier;
        // make sure arg passed exactly aligns with type specified
        // deref coerced match is allowed
        // non-binding assign, won't take ownership but introduce compile time type check
        #(let _: #args_ty = #args_val;)*
        // this boxed raw pointer will be dropped in interface `transumte_then_invoke`
        let boxed_opt_args_ptr = alloc::boxed::Box::into_raw(
            alloc::boxed::Box::<(#(Option<#args_ty>,)*)>::new((#(Some(#args_val),)*))
        ) as *const ();
        let ret_place_holder = alloc::boxed::Box::into_raw(
            alloc::boxed::Box::new(Some(core::mem::MaybeUninit::<#ret_ty>::uninit()))
        ) as *mut ();
        ::trusted_kernel::invoke_proxy::<#ret_ty>(api_identifier, boxed_opt_args_ptr, ret_place_holder)
    }}.into()
}
