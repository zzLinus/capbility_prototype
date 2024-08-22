use alloc::boxed::Box;

use crate::sync::Mutex;
use crate::warn;
use alloc::collections::BTreeMap;
use alloc::string::String;
use core::any::TypeId;
use lazy_static::lazy_static;
use linkme;
use spin::rwlock::RwLock;

#[linkme::distributed_slice]
pub static API_REGISTRY: [fn() -> Box<dyn GlobalInterface + Send + Sync>];

lazy_static! {
    /// use RwLock for this to avoid dead lock in chained `invoke_proxy` call
    pub static ref GLOBAL_API_RESERVOIR: RwLock<BTreeMap<ExportedAPIIdentifier, Box<dyn GlobalInterface + Send + Sync>>> = {
        let mut api_reservoir = BTreeMap::new();
        for interface_proxy in API_REGISTRY {
            let interface = interface_proxy();
            api_reservoir.insert(interface.get_identifier(), interface);
        }
        RwLock::new(api_reservoir)
    };
}

/// # Safety
/// caller of `fn invoke()` should guarantee that data and return value place holder passed into invoke is obtained from Box::into_raw() with correct type
pub trait GlobalInterface {
    fn path(&self) -> String;
    fn transmute_then_invoke(&self, data: *const (), ret_place_holder: *mut ());
    fn get_identifier(&self) -> ExportedAPIIdentifier;
}

#[derive(PartialOrd, PartialEq, Eq, Ord, Debug)]
pub struct ExportedAPIIdentifier {
    api_name: String,
    args_hash: TypeId,
    ret_hash: TypeId,
}

impl ExportedAPIIdentifier {
    pub fn new<T: alloc::string::ToString>(
        api_name: T,
        args_hash: TypeId,
        ret_hash: TypeId,
    ) -> Self {
        Self {
            api_name: api_name.to_string(),
            args_hash,
            ret_hash,
        }
    }
}

/// typical workflow of safeos_export
/// the maintainer (usually one who publish the linkme distributed slice) bookkeeps the mapping
/// between exported API identifier: currently it is a triplet of {exported_name, type hash of args tupe, type hash of return type}
/// and the global interface trait object
/// if such interface is successfully identified, safeos_invoke! macro will allocate boxed memory for parameter passing and return data retrieve
/// and call GlobalInterface::transmute_then_invoke
/// move boxed raw pointer reclaim in invoke_proxy to prevent disseminate unsafe code to invoker crate
pub fn invoke_proxy<T>(
    api_identifier: ExportedAPIIdentifier,
    arg: *const (),
    ret: *mut (),
) -> Option<T> {
    GLOBAL_API_RESERVOIR
        .read()
        .get(&api_identifier)
        .and_then(|interface| unsafe {
            crate::catch_unwind(|| {
                interface.transmute_then_invoke(arg, ret);
            })
            .ok()
            .and_then(|_| Box::<Option<T>>::from_raw(ret as *mut Option<T>).take())
        })
}
