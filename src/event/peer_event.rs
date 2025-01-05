use crate::addr::ipv6_scope::Ipv6Scope;
use crate::env::uid::Uid;
use crate::msg::msg::Message;
use crate::protocol::peer_ctrl_code::PeerCtrlCode;
// 无丢失状态
// 请求时验证

// 宏生成
#[derive(Clone)]
pub enum PeerEvent {
    HELLO(Uid, Ipv6Scope),
    CONNECTED(Uid, Ipv6Scope),
    ESTABLISHED(Uid,Ipv6Scope),
    TRANSFERRING(Uid),
    UNREACHABLE(Uid),
}

unsafe impl Sync for PeerEvent {}
unsafe impl Send for PeerEvent {}

impl From<Message> for PeerEvent {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Hello(hello_msg) =>
                PeerEvent::HELLO(hello_msg.host_id, hello_msg.addr),
            Message::Ctrl(ctrl_msg) => match ctrl_msg.ctrl_code {
                n if PeerCtrlCode::CONNECT.bits() == n => PeerEvent::CONNECTED(ctrl_msg.host_id, ctrl_msg.addr),
                _ => { panic!("Unexpected message code"); }
            }
        }
    }
}
