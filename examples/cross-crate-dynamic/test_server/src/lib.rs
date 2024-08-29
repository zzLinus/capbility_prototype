#![no_std]

#[service_loader::export_interface]
fn test(a: usize, b: usize) -> usize {
    a + b
}
