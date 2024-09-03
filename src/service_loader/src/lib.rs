#![no_std]
pub use crate_export_macro::export_interface;
pub use crate_import_macro::generate_interface;

generate_interface!(kernel,load_crate,crate_name : &str,crate_data : &[u8] => ());

#[export_interface]
fn load_crate(crate_name: &str, crate_data: &[u8]) {
    kernel_load_crate(crate_name, crate_data);
}
