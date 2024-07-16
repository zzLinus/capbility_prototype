mod capability;

use crate::capability::cap::*;

fn main() {
    let uc = Cap::create_new(capability::cap::CapType::Untyped);
    let ec = Cap::create_new(capability::cap::CapType::EndPoint);
    let ec2 = Cap::create_new(capability::cap::CapType::EndPoint);
    let container = vec![uc.clone(), ec.clone(), ec2.clone()];

    for c in container {
        c.as_ref()
            .as_ref()
            .unwrap()
            .lock()
            .expect("REASON")
            .decode_capinvok(CapInvLable::RETYPE);
    }

    println!("finish")
}
