mod capability;
use capability::structs::TCB;

use crate::capability::cap::*;

fn main() {
    let mut tcb = Box::new(TCB::new());
    let uc1 = Cap::get_root_untpye();

    tcb.ipc_buf.mrs[0] = 0; // NOTE: Make a new untype
    Option::as_ref(&uc1.0)
        .unwrap()
        .lock()
        .expect("REASON")
        .decode_capinvok(CapInvLable::RETYPE, tcb.as_ref());

    //NOTE: get the last children which is the untype just created
    let uc2 = Option::as_ref(&uc1.0)
        .unwrap()
        .lock()
        .expect("REASON")
        .get_new_child();

    tcb.ipc_buf.mrs[0] = 1; // NOTE: Make a new EndPoint
    Option::as_ref(&uc2)
        .unwrap()
        .lock()
        .expect("REASON")
        .decode_capinvok(CapInvLable::RETYPE, tcb.as_ref());

    //NOTE: get the last children which is the untype just created
    let ec1 = Option::as_ref(&uc2)
        .unwrap()
        .lock()
        .expect("REASON")
        .get_new_child();

    //tcb.ipc_buf.mrs[0] = 1; // NOTE: No specific argument needed
    Option::as_ref(&ec1) // using endpoint cap to invoke kobj funcition
        .unwrap()
        .lock()
        .expect("REASON")
        .decode_capinvok(CapInvLable::NB_SEND, tcb.as_ref());

    println!("finish")
}
