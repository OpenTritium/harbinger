use crate::addr_v6::Ipv6Scope;
use crate::msg::{Msg, Parcel};
use crate::utils::Uid;

// 派生宏支持
#[derive(Clone)]
pub enum PeerEvent {
    Hello { host_id: Uid, addr: Ipv6Scope },
    Connect { host_id: Uid, addr: Ipv6Scope },
    Conflict { addr: Ipv6Scope },
    Established, // 仅作日志记录
}

impl From<Parcel> for PeerEvent {
    fn from(val: Parcel) -> Self {
        match val {
            (Msg::Hello { host_id, addr }, _) => PeerEvent::Hello { host_id, addr }, // 忽略广播地址
            (Msg::Connect { host_id }, addr) => PeerEvent::Connect { host_id, addr },
            (Msg::Conflict, addr) => PeerEvent::Conflict { addr },
        }
    }
}

unsafe impl Sync for PeerEvent {}
unsafe impl Send for PeerEvent {}
