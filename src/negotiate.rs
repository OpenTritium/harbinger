use super::env::{Env, get_env};
use crate::addr::addr::Ipv6Scope;
use crate::addr::addr::Ipv6Scope::LinkLocal;
use crate::msg::hello::HelloMsg;
use crate::msg::msg::Message;
use crate::msg::opt::{OptCode, OptMsg};
use crate::peer::future_state::PeerFutureState;
use crate::uid::Uid;
use dashmap::DashMap;
use dashmap::mapref::one::RefMut;
use std::net::{SocketAddr, SocketAddrV6};
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::UdpSocket;

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
                a @ LinkLocal(_, _) => Some(a),
                _ => None,
            })
            .ok_or(DiscoveryError::BestMetricNicNotFound)?;
        let sockaddr_local_link = host.clone().into_sockaddr_v6(*p);
        let s = UdpSocket::bind(&sockaddr_local_link).await.unwrap();
        s.join_multicast_v6(mla, scope_id).unwrap();
        // todo 处理全球
        Ok(Discovery {
            link_local_sockaddr: sockaddr_local_link,
            socket: s,
        })
    }
    // todo id冲突解决
    pub async fn hello(&self) {
        let h = HelloMsg::new(&Ipv6Scope::try_from_sockaddr_v6(self.link_local_sockaddr).unwrap())
            .to_string();
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
    pub async fn listen(&self, peers: Arc<DashMap<Uid, PeerFutureState>>) {
        let mut buffer = [0u8; 1024];
        let (len, src) = self.socket.recv_from(&mut buffer).await.unwrap();
        let msg = String::from_utf8_lossy(&buffer[..len]);
        dbg!(&msg);
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
                    peers.insert(uid, pfs);
                    dbg!(peers);
                }
                // established 后不要轻举妄动
                Message::Opt(o) => match OptCode::from_bits(o.opt_code).unwrap().bits() {
                    x if x == OptCode::CONNECTING.bits() => {
                        peers.insert(
                            o.host_id,
                            PeerFutureState::Establish(Ipv6Scope::try_from_sockaddr_v6(a).unwrap()),
                        );
                    }
                    x if x == OptCode::ESTABLISHING.bits() => {}
                    _ => {}
                },
            }
        }
    }
    // 无状态发送
    pub async fn select(&self, mut record: RefMut<'_, Uid, PeerFutureState>) {
        let port = get_env().port;
        let state = record.value();
        match state {
            PeerFutureState::Connect(addr) => {
                let connect_msg = OptMsg::gen_msg_by_state(&state).to_string();
                // todo 处理错误
                self.socket
                    .send_to(connect_msg.as_bytes(), addr.clone().into_sockaddr_v6(port))
                    .await
                    .unwrap();
                *record.value_mut() = PeerFutureState::Establish(addr.clone());
            }
            PeerFutureState::Establish(_) => {
                // 发出或收到 connect 消息后保持此状态
            }
            PeerFutureState::Dispose(_) => todo!(),
            PeerFutureState::Disconnect(_) => todo!(),
        }
    }
}
