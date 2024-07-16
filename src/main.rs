mod capability;
use capability::structs::TCB;

use crate::capability::cap::*;

fn main() {
    let mut tcb = Box::new(TCB::new());
    let uc1 = Cap::get_root_untpye();

    tcb.ipc_buf.mrs[0] = 0; // NOTE: Make a new untype
    uc1.as_ref()
        .as_ref()
        .unwrap()
        .lock()
        .expect("REASON")
        .decode_capinvok(CapInvLable::RETYPE, tcb.as_ref());

    uc1 //NOTE: get the last children which is the untype just created
        .as_ref()
        .as_ref()
        .unwrap()
        .lock()
        .expect("REASON")
        .get_new_child();

    //for c in container {
    //    c.as_ref()
    //        .as_ref()
    //        .unwrap()
    //        .lock()
    //        .expect("REASON")
    //        .decode_capinvok(CapInvLable::RETYPE);
    //}

    println!("finish")
}
