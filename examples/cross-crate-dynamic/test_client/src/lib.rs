#![no_std]

crate_import_macro::generate_interface!(test_server,test,a: usize,b:usize=>usize);

#[no_mangle]
fn call_test_server_test() -> usize {
    test_server_test(1, 2)
}
