#![deny(clippy::perf, clippy::complexity)]

use core::alloc::Layout;
use core::borrow::BorrowMut;
use core::intrinsics::mul_with_overflow;
use core::ops::{Deref, DerefMut};

use super::cdt::CdtNode;
use super::object::KObj;
use super::rights::{self, Rights};
use crate::kernel_object::{page_table, Frame, PageTable, Untyped, TCB};
use crate::kernel_object::{RetypeErr, RetypeInit};
use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use spin::{Mutex, MutexGuard};
#[derive(Clone)]
pub struct Cap {
    pub inner: Weak<Mutex<CapInner>>,
}
pub struct CapInner {
    pub object: Arc<Mutex<KObj>>,
    pub rights: Rights,
    pub cdt_node: Weak<Mutex<CdtNode>>,
}

#[derive(Debug)]
pub enum CapErr {
    MintErr,
    UpgradeErr,
    RetypeErr,
    DecodeErr,
}

impl Cap {
    pub fn new(
        object: Arc<Mutex<KObj>>,
        rights: Rights,
        cdt_node: Weak<Mutex<CdtNode>>,
    ) -> (Cap, Arc<Mutex<CapInner>>) {
        let mut cap = Cap { inner: Weak::new() };
        let inner = Arc::new(Mutex::new(CapInner {
            object,
            rights,
            cdt_node,
        }));
        cap.inner = Arc::downgrade(&inner);
        (cap, inner)
    }

    fn new_cap(&self, kobj: KObj) -> (Cap, Arc<Mutex<CapInner>>) {
        let mut cap = Cap { inner: Weak::new() };
        let inner = Arc::new(Mutex::new(CapInner {
            object: Arc::new(Mutex::new(kobj)),
            rights: Rights::default(),
            cdt_node: Weak::new(),
        }));
        cap.inner = Arc::downgrade(&inner);
        (cap, inner)
    }

    fn mint_cap(&self, kobj: Arc<Mutex<KObj>>, rights: Rights) -> (Cap, Arc<Mutex<CapInner>>) {
        let mut cap = Cap { inner: Weak::new() };
        let inner = Arc::new(Mutex::new(CapInner {
            object: kobj,
            rights: rights,
            cdt_node: Weak::new(),
        }));
        cap.inner = Arc::downgrade(&inner);
        (cap, inner)
    }

    pub fn get_inner(&self) -> Result<Arc<Mutex<CapInner>>, CapErr> {
        self.inner.upgrade().ok_or_else(|| CapErr::UpgradeErr)
    }

    pub fn revoke(&self) -> Result<(), CapErr> {
        self.get_inner()?
            .lock()
            .cdt_node
            .upgrade()
            .ok_or(CapErr::UpgradeErr)?
            .lock()
            .revoke();
        Ok(())
    }

    pub fn mint(&self, rights: Rights) -> Result<Cap, CapErr> {
        let kobj = self.get_inner()?.lock().object.clone();

        let (cap, inner) = self.mint_cap(kobj, rights);

        let cdt = Arc::new(Mutex::new(CdtNode::new(inner)));
        cap.get_inner()?.lock().cdt_node = Arc::downgrade(&cdt);
        // append new cap's cdt_node to father->child
        self.get_inner()?
            .lock()
            .cdt_node
            .upgrade()
            .ok_or(CapErr::UpgradeErr)?
            .lock()
            .child
            .push(cdt.clone());
        Ok(cap)
    }

    pub fn retype<T>(&self) -> Result<Cap, CapErr>
    where
        T: RetypeInit,
        T::StoredAs: Sized,
    {
        //let mut kobj_guard = &self.get_inner().lock().object;
        let new_obj = match *self.get_inner()?.lock().object.lock() {
            KObj::Untyped(ref mut untyped) => untyped.retype::<T>().unwrap(),
            _ => panic!("retype can only be performed on an untyped object"),
        };

        let (cap, inner) = self.new_cap(new_obj);

        let cdt = Arc::new(Mutex::new(CdtNode::new(inner)));
        cap.get_inner()?.lock().cdt_node = Arc::downgrade(&cdt);
        // append new cap's cdt_node to father->child
        self.get_inner()?
            .lock()
            .cdt_node
            .upgrade()
            .ok_or(CapErr::UpgradeErr)?
            .lock()
            .child
            .push(cdt.clone());
        Ok(cap)
    }

    pub fn retype_dyn_sized<T: RetypeInit>(&self, size: usize) -> Result<Cap, CapErr> {
        let new_obj = match *self.get_inner()?.lock().object.lock() {
            KObj::Untyped(ref mut untyped) => untyped.retype_dyn_sized::<T>(size).unwrap(),
            _ => panic!("retype can only be performed on an untyped object"),
        };

        let (cap, inner) = self.new_cap(new_obj);

        let cdt = Arc::new(Mutex::new(CdtNode::new(inner)));
        cap.get_inner()?.lock().cdt_node = Arc::downgrade(&cdt);
        //append new cap's cdt_node to father->child
        self.get_inner()?
            .lock()
            .cdt_node
            .upgrade()
            .ok_or(CapErr::UpgradeErr)?
            .lock()
            .child
            .push(cdt.clone());
        Ok(cap)
    }
}
