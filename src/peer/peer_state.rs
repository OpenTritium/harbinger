use crate::addr::ipv6_scope::Ipv6Scope;

#[derive(Eq, Hash, PartialEq, Debug)]
#[derive(Clone)]
// 无丢失状态
// 请求时验证

// 宏生成
pub enum PeerState {
    Connect(Ipv6Scope),       // 要求对方知道自己的存在
    Establish(Ipv6Scope), //双方都了解状态
}



