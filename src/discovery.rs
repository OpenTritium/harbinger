use crate::addr::ipv6_scope::Ipv6Scope;
use crate::addr::ipv6_scope::Ipv6Scope::LinkLocal;
use crate::env::env::{get_env, Env};
use crate::env::uid::Uid;
use crate::msg::hello_msg::HelloMsg;
use crate::msg::msg::Message;
use crate::msg::opt_msg::OptMsg;
use crate::peer::peer_state::PeerState;
use crate::peer::peer_state_code::PeerStateCode;
use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{SocketAddr, SocketAddrV6};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::net::UdpSocket;
use tokio::time::interval;
use futures::stream::{self,StreamExt};

pub struct Discovery {
    link_local_sockaddr: SocketAddrV6,
    socket: UdpSocket,
}

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("")]
    BestMetricNicNotFound,
    #[error("")]
    LocalLinkNotFound,
    #[error("")]
    RouteUnavailable,
}

//发现服务是特例，要生产peer记录，anchor才有得查
impl Discovery {
    // todo!  错误传播
    // todo! global 支持
    // 初始化socket
    pub async fn new() -> Result<Discovery, DiscoveryError> {
        if !get_env().is_ipv6_route_available() {
            return Err(DiscoveryError::RouteUnavailable);
        }
        let Env {
            multicast_global,
            multicast_local,
            nics,
            ..
        } = get_env();
        // pick the best nic by metric
        let (&scope_id, (_, addrs)) = nics.iter().min_by_key(|r| r.0).unwrap();
        let host = addrs
            .iter()
            .rev()
            .find_map(|addr| match addr {
                a @ LinkLocal(_, _) => Some(a),
                _ => None,
            })
            .ok_or(DiscoveryError::BestMetricNicNotFound)?;
        let sockaddr_local_link = host.clone().into_sockaddr_v6();
// 去除 socket 2
        let s = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP)).unwrap();
        s.set_reuse_address(true).unwrap();
        s.bind(&sockaddr_local_link.clone().into()).unwrap();
        s.join_multicast_v6(multicast_local, scope_id).unwrap();
        // todo 处理全球
        Ok(Discovery {
            link_local_sockaddr: sockaddr_local_link,
            socket: UdpSocket::from_std(s.into()).unwrap(),
        })
    }
    // todo id冲突解决
    pub async fn hello(&self) {
        let h = HelloMsg::new(&Ipv6Scope::try_from_sockaddr_v6(self.link_local_sockaddr).unwrap())
            .to_string();
        //dbg!(&h);
        let Env {
            multicast_local,
            port,
            ..
        } = get_env();
        if let Ok(bytes_sent) = self
            .socket
            .send_to(
                h.as_bytes(),
                SocketAddrV6::new(
                    *multicast_local,
                    *port,
                    0,
                    self.link_local_sockaddr.scope_id(),
                ),
            )
            .await
        {
            //dbg!(bytes_sent);
        }
    }
    //我提出 greet,unicast

    // todo 统一 buffer
    // 读写监控socket
    pub async fn listen(&self, peers: Arc<DashMap<Uid, PeerState>>) {

        let mut stream = UdpSocketRecvFrom::new(&self.socket);
        let mut buffer = [0u8; 1024];
        if let Ok((len, src)) = self.socket.recv_from(&mut buffer).await {
            let msg = Message::from_str(&String::from_utf8_lossy(&buffer[..len])).unwrap();
            if let SocketAddr::V6(a) = src {
                match msg {
                    // 收到hello后将要connect
                    Message::Hello(hello) => {
                        //todo filter 本机
                        // 收到 问候后 greet
                        // 然后生成对应状态的peer
                        let (uid, pfs) = hello.into();
                        if uid != get_env().host_id {
                            peers.insert(uid, pfs);
                        }
                        //dbg!(peers);
                    }
                    Message::Opt(o) => match PeerStateCode::from_bits(o.opt_code).unwrap().bits() {
                        x if x == PeerStateCode::CONNECTING.bits() => {
                            peers.insert(
                                o.host_id,
                                PeerState::Establish(
                                    Ipv6Scope::try_from_sockaddr_v6(a).unwrap(),
                                ),
                            );
                        }
                        // established 后不要轻举妄动
                        x if x == PeerStateCode::ESTABLISHED.bits() => {}
                        _ => {}
                    },
                }
            } else {
                return;
            }
        }
    }
}
