#![no_std]
#[macro_export]
macro_rules! generate_interface {
    ($crate_name:ident, $function_name:ident, $($arg_name :ident :$arg_ty:ty),* => $ret:ty) => {
        paste::item! {
            #[allow(unused)]
            #[no_mangle]
            #[allow(non_upper_case_globals)]
            #[link_section= ".extern_code"]
            static [<$crate_name ___ $function_name>]: fn($($arg_ty),*) -> $ret = {
                fn default_impl($(_: $arg_ty),*) -> $ret {
                    panic!(concat!(stringify!($crate_name), "::", stringify!($function_name), " is not implemented, should be linked by kernel when loaded."));
                }
                default_impl
            };
            #[no_mangle]
            fn [<$crate_name _ $function_name>]($($arg_name: $arg_ty),*) -> $ret {
                let func = unsafe {
                    core::ptr::read_volatile(&[<$crate_name ___ $function_name>])
                };
                func($($arg_name),*)
            }
        }
    };
}
