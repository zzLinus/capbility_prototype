#![no_std]
extern crate alloc;
use alloc::boxed::Box;
use cross_crate::GlobalInterface;
use cross_crate::GLOBAL_INTERFACES;
#[linkme::distributed_slice(GLOBAL_INTERFACES)]
static TO_EXPORTS: fn() -> Box<dyn GlobalInterface> = func_spawn_handle1;

pub fn spawn() -> usize {
    let test_res = 20;
    test_res
}

#[no_mangle]
pub fn func_spawn_handle1() -> Box<dyn GlobalInterface> {
    Box::new(FuncSpawn)
}

struct FuncSpawn;

impl GlobalInterface for FuncSpawn {
    fn name(&self) -> &'static str {
        "spawn"
    }
    fn crate_name(&self) -> &'static str {
        "test_client"
    }
    fn func(&self) -> usize {
        spawn as usize
    }
}
