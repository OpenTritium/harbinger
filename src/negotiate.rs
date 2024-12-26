use super::env::{get_env, Env};
use crate::msg::hello::HelloMsg;
use crate::msg::msg::Message;
use crate::msg::opt::{OptCode, OptMsg};
use crate::peer::PeerFutureState;
use dashmap::DashMap;
use std::net::{SocketAddr, SocketAddrV6};
use std::str::FromStr;
use thiserror::Error;
use tokio::net::UdpSocket;
use crate::addr::addr::Ipv6Scope;
use crate::addr::addr::Ipv6Scope::LinkLocal;
use crate::uid::Uid;

pub struct Discovery {
    peers: DashMap<Uid,PeerFutureState>,
    link_local_sockaddr:SocketAddrV6,
    socket: UdpSocket,
}

#[derive(Debug,Error)]
pub enum DiscoveryError{
    #[error("")]
    BestMetricNicNotFound,
    #[error("")]
    LocalLinkNotFound,
    #[error("")]
    RouteUnavailable,
}

// todo! 向任务管理器传递 rx
impl Discovery {
    // todo!  错误传播
    // todo! global 支持
    // 初始化socket
    pub async fn new() -> Result<Discovery, DiscoveryError> {
        if !get_env().is_ipv6_route_available() {
            return Err(DiscoveryError::RouteUnavailable);
        }
        let Env {
            multicast_global: mga,
            multicast_local: mla,
            port: p,
            nics: ns,
            ..
        } = get_env();
        // select the best nic
        let (&scope_id, (_, addrs)) = ns.iter().min_by_key(|r| r.0).unwrap();
        let host = addrs
            .iter()
            .rev()
            .find_map(|addr| match addr {
                a @ LinkLocal(_,_) => Some(a),
                _ => None,
            })
            .ok_or(DiscoveryError::BestMetricNicNotFound)?;
        let sockaddr_local_link = host.clone().into_sockaddr_v6(get_env().port);
        let s = UdpSocket::bind(&sockaddr_local_link).await.unwrap();
        s.join_multicast_v6(mla, scope_id).unwrap();
        // todo 处理全球
        Ok(Discovery {
            peers: DashMap::new(),
            link_local_sockaddr:sockaddr_local_link,
            socket: s,
        })
    }
    // todo id冲突解决
    pub async fn hello(&self) {
        let h = HelloMsg::new(&Ipv6Scope::try_from_sockaddr_v6(self.link_local_sockaddr).unwrap()).to_string();
        dbg!(&h);
        let Env {
            multicast_local: mla,
            port: p,
            ..
        } = get_env();
        if let Ok(bytes_sent) = self
            .socket
            .send_to(h.as_bytes(), self.link_local_sockaddr)
            .await
        {
            dbg!(bytes_sent);
        }
    }
    //我提出 greet,unicast

    // todo 统一 buffer
    // 读写监控socket
    pub async fn on_discovering(&self) {
        let mut buffer = [0u8; 1024];
        let (len, src) = self.socket.recv_from(&mut buffer).await.unwrap();
        let msg = String::from_utf8_lossy(&buffer[..len]);
        let m = Message::from_str(&msg).unwrap();
        if let SocketAddr::V6(a) = src {
            dbg!(&m);
            // peek 再拿
            match m {
                // 收到hello后将要connect
                Message::Hello(hello) => {
                    //todo filter 本机
                    // 收到 问候后 greet
                    // 然后生成对应状态的peer
                    let (uid, pfs) = hello.into();
                    self.peers.insert(uid, pfs);
                    dbg!(&self.peers);
                }
                // established 后不要轻举妄动
                Message::Opt(o) => match OptCode::from_bits(o.opt_code).unwrap().bits() {
                    x if x == OptCode::CONNECTING.bits() => {
                        self.peers.insert(o.host_id, PeerFutureState::Establish(Ipv6Scope::try_from_sockaddr_v6(a).unwrap()));
                    },
                    x if x == OptCode::ESTABLISHING.bits() => {},
                    _ => {}
                    
                }
            }
        }
    }
    // 无状态发送
    pub async fn poll_peers(&self) {
        
        for record in self.peers.iter_mut() {
            match record.value() {
               m@ PeerFutureState::Connect(c) => {
                   let msg  = OptMsg::gen_msg_by_enum(&m).to_string();
                   let r = self.socket.send_to(msg.as_bytes(), self.link_local_sockaddr).await.unwrap();
                },
               &PeerFutureState::Establish(_) | &PeerFutureState::Dispose(_) | &PeerFutureState::Disconnect(_) => todo!()
            }
        }
    }
}
