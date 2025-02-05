use crate::addr_v6::{Ipv6Scope, ScopeWithPort};
use crate::msg::codec::MsgCodec;
use crate::msg::msg::Msg;
use crate::utils::{env, nic_selected};
use anyhow::{Result, anyhow};
use futures::stream::{SplitSink, SplitStream};
use netif::Interface;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{SocketAddr, SocketAddrV6};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio_util::udp::UdpFramed;

pub type MsgSink = SplitSink<UdpFramed<MsgCodec>, (Msg, SocketAddr)>;
pub type MsgStream = SplitStream<UdpFramed<MsgCodec>>;
pub type Parcel = (Msg, Ipv6Scope);
pub type ParcelSender = Sender<Parcel>;
pub type ParcelReceiver = Receiver<Parcel>;

pub struct ProtocolSocketFactory;
type LanWanSocket = [Result<UdpSocket>; 2];

// 响应网络切换事件
impl ProtocolSocketFactory {
    //todo  去重
    fn new_wan_sock(iface: &Interface) -> Result<UdpSocket> {
        let sock_addr: SocketAddrV6 =
            ScopeWithPort::new(iface.try_into()?, env().protocol_port).into();
        let sock = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
        
        sock.set_only_v6(true)?;
        sock.bind(&sock_addr.into())?;
        sock.join_multicast_v6(&env().multicast_wan, 0)?;
        sock.set_multicast_loop_v6(false)?;
        let sock = UdpSocket::from_std(sock.into())?;
        Ok(sock)
    }

    fn new_lan_sock(iface: &Interface) -> Result<UdpSocket> {
        let scope_id = iface.scope_id().ok_or(anyhow!("scope id is not exsits"))?;
        let sock_addr: SocketAddrV6 =
            ScopeWithPort::new(iface.try_into()?, env().protocol_port).into();
        let sock = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
        sock.set_only_v6(true)?;
        sock.bind(&sock_addr.into())?;
        
        sock.join_multicast_v6(&env().multicast_lan, scope_id)?;
        sock.set_multicast_loop_v6(false)?;
        let sock = UdpSocket::from_std(sock.into())?;
        Ok(sock)
    }

    pub async fn new() -> LanWanSocket {
        let [lan, wan] = nic_selected();
        [
            lan.as_ref().map_or_else(
                || Err(anyhow!("Lan interface is not available.")),
                Self::new_lan_sock,
            ),
            wan.as_ref().map_or_else(
                || Err(anyhow!("Wan interface is not available.")),
                Self::new_wan_sock,
            ),
        ]
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[tokio::test(flavor = "multi_thread")]
//     async fn try_init() {}
// }
