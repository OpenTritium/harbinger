use crate::addr_v6::Ipv6Scope;
use crate::msg::{Msg, Parcel};
use crate::utils::Uid;

use super::PeerEventFlags;

// 派生宏支持
#[derive(Clone, Debug)]
pub enum PeerEvent {
    Hello { host_id: Uid, addr: Ipv6Scope },
    Connect { host_id: Uid, addr: Ipv6Scope },
    Conflict { addr: Ipv6Scope },
    Established, //需要注意的是，永远不可能有 Parcel 转换到该事件，该事件仅用于自循环
}

impl From<Parcel> for PeerEvent {
    fn from(val: Parcel) -> Self {
        match val {
            (Msg::Hello { host_id, mut addr }, multi) => PeerEvent::Hello {
                host_id,
                addr: {
                    addr.modify_scope_id(multi.scope_id());
                    addr
                },
            },
            (Msg::Connect { host_id }, addr) => PeerEvent::Connect { host_id, addr },
            (Msg::Conflict, addr) => PeerEvent::Conflict { addr },
        }
    }
}

impl From<&PeerEvent> for u8 {
    fn from(val: &PeerEvent) -> Self {
        match val {
            PeerEvent::Hello { .. } => PeerEventFlags::HELLO.bits(),
            PeerEvent::Connect { .. } => PeerEventFlags::CONNECT.bits(),
            PeerEvent::Conflict { .. } => PeerEventFlags::CONFLICT.bits(),
            PeerEvent::Established => PeerEventFlags::ESTABLISHED.bits(),
        }
    }
}

unsafe impl Sync for PeerEvent {}
unsafe impl Send for PeerEvent {}
