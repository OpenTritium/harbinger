use crate::addr_v6::scope::Ipv6Scope;
use crate::env::uid::Uid;
use crate::msg::msg::Msg;
use crate::protocol::peer_ctrl_code::PeerCtrlCode;
// todo 请求时验证

// todo 宏生成
#[derive(Clone)]
pub enum PeerEvent {
    HELLO { host_id: Uid, addr: Ipv6Scope },
    CONNECTED { host_id: Uid, addr: Ipv6Scope },
    ESTABLISHED { host_id: Uid, addr: Ipv6Scope },
    TRANSFERRING(Uid),
    UNREACHABLE(Uid),
}

unsafe impl Sync for PeerEvent {}
unsafe impl Send for PeerEvent {}

// todo
impl From<Msg> for PeerEvent {
    fn from(msg: Msg) -> Self {
        match msg {
            Msg::Hello(hello_msg) => PeerEvent::HELLO {
                host_id: hello_msg.host_id,
                addr: hello_msg.addr,
            },
            Msg::Ctrl(ctrl_msg) => match ctrl_msg.ctrl_code {
                n if PeerCtrlCode::CONNECT.bits() == n => PeerEvent::CONNECTED {
                    host_id: ctrl_msg.host_id,
                    addr: ctrl_msg.addr,
                },
                _ => {
                    panic!("Unexpected message code");
                }
            },
        }
    }
}
