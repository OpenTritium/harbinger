use crate::addr::addr::Ipv6Scope;
use crate::msg::hello::HelloMsg;
use PeerFutureState::Connect;
use crate::uid::Uid;

#[derive(Eq, Hash, PartialEq, Debug)]
#[derive(Clone)]
pub enum PeerFutureState {
    Connect(Ipv6Scope),       // 发送 greet 报文后保持
    Establish(Ipv6Scope), //确认后保持
    Dispose(Ipv6Scope),
    Disconnect(Ipv6Scope),
}

impl From<HelloMsg> for (Uid, PeerFutureState) {
    fn from(msg: HelloMsg) -> Self {
        (msg.host_id, Connect(msg.addr))
    }
}

