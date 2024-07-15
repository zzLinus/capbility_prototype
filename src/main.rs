mod capability;

use crate::capability::cap::*;

fn main() {
    let uc = Cap::create_new(capability::cap::CapType::Untyped);
    let ec = Cap::create_new(capability::cap::CapType::EndPoint);
    let container = vec![uc, ec];

    for c in container {
        c.decode_capinvok(CapInvLable::RETYPE);
    }

    println!("finish")
}
