use super::PeerEventFlags;
use crate::addr_v6::Ipv6Scope;

pub type StateTable = dashmap::DashMap<crate::utils::Uid, StateEntry>;

#[derive(Debug)]
pub struct StateEntry {
    addr: Ipv6Scope,
    pub state: PeerEventFlags,
    shared_key: Option<u8>,
}

impl StateEntry {
    // 记得蜜月
    pub fn new(addr: Ipv6Scope, init_state: PeerEventFlags) -> Self {
        Self {
            addr,
            state: init_state,
            shared_key: None,
        }
    }
}
