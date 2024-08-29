#![no_std]

#[crate_export_macro::export_interface]
fn test(a: usize, b: usize) -> usize {
    a + b
}
