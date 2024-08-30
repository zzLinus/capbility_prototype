# README

this directory contains examples of cross-crate-dynamic. the kernel services communicate with each other by using the macro crates `crate_import_macro` and `crate_export_macro`. the first one provides macro `generate_interface` to import other kernel services' exported functions while the latter one provides macro `export_interface` to export services' functions. the static variable binding is done by kernel when the service is loaded. The kernel is transplant to kernel services.

## Usage

1. get the service's binary data by include_bytes! or other methods
2. get the corrosponding namespace(kernel) and call the load_crate function with the crate_name and crate_data
3. get the function's addr you want to call by calling the namespace's get_func function
4. turn the vaddr into function pointer and just call it.

```rust
crate_mgmt::init();
let kernel_namespace = crate_mgmt::get_kernel_namespace().unwrap();
load_crate(kernel_namespace.clone(), "test_server", SERVER).unwrap();
load_crate(kernel_namespace.clone(), "test_client", CLIENT).unwrap();
let test_func_addr = kernel_namespace.get_func("test_server", "test").unwrap();
unsafe {
    let test_func: fn(usize, usize) -> usize = transmute(test_func_addr);
    println!("the return value is {}", test_func(1, 2));
}
```