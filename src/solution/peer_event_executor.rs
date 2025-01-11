use crate::addr_v6::scope::Ipv6Scope;
use crate::env::uid::Uid;
use crate::event::peer_event::PeerEvent;
use crate::interceptor::msg_interceptor::msg_interceptor;
use crate::msg::msg::Msg;
use crate::msg::protocol_socket::ProtocolSocket;
use crate::protocol::peer_ctrl_code::PeerCtrlCode;
use crate::solution::peer_event_solution::PeerEventSolution;
use dashmap::DashMap;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use tokio::sync::mpsc::Sender;
use tracing::info;

pub struct PeerEventExecutor {
    pub peers: Arc<DashMap<Uid, Ipv6Scope>>,
    solution_mapping: Arc<DashMap<u8, Vec<&'static dyn PeerEventSolution>>>,
    sender: Sender<(Msg, SocketAddr)>, // todo 可以减少一次clone
}

pub static PEER_EVENT_EXECUTOR: OnceLock<PeerEventExecutor> = OnceLock::new();

pub fn peer_event_executor() -> &'static PeerEventExecutor {
    PEER_EVENT_EXECUTOR.get_or_init(PeerEventExecutor::new)
}
impl PeerEventExecutor {
    pub fn new() -> Self {
        info!("事件执行器初始化");
        Self {
            peers: Arc::new(DashMap::new()),
            solution_mapping: Arc::new(
                [
                    (PeerCtrlCode::HELLO.bits(), vec![]),
                    (PeerCtrlCode::CONNECT.bits(), vec![]),
                    (PeerCtrlCode::ESTABLISH.bits(), vec![]),
                    (PeerCtrlCode::TRANSFERRING.bits(), vec![]),
                    (PeerCtrlCode::UNREACHABLE.bits(), vec![]),
                ]
                .into_iter()
                .collect(),
            ),
            sender: ProtocolSocket::get_sender(),
        }
    }
    // 上游消息流
    // 多播只会收到hello
    //单播收到opt

    pub async fn listening(&self) {
        let (loopback, mut listener) = msg_interceptor().bridge_and_filtering().await;
        let peers = self.peers.clone();
        let solution_mapping = self.solution_mapping.clone();
        let sender = self.sender.clone();
        tokio::spawn(async move {
            // 消费一定要跟上速度，不然通道会被挤爆的
            loop {
                let event = listener.recv().await.unwrap();
                match event {
                    // 收到 hello 准备回复
                    e @ PeerEvent::HELLO { .. } => {
                        info!("触发 hello 事件处理");
                        if let Some(v) = solution_mapping.get(&PeerCtrlCode::HELLO.bits()) {
                            for s in v.iter() {
                                info!("找到事件处理器");
                                let closure = s.dispatch_solution(e.clone().into()).unwrap();
                                closure(sender.clone(), peers.clone(), loopback.clone())
                                    .await
                                    .unwrap();
                            }
                        }
                    }
                    // 如果表中有此用户，不插入，如果没有就插入
                    PeerEvent::CONNECTED { host_id, addr } => {
                        info!("触发连接");
                        loopback
                            .send(PeerEvent::ESTABLISHED {
                                host_id: host_id.clone(),
                                addr: addr.clone(),
                            })
                            .await
                            .unwrap();
                        peers
                            .entry(host_id)
                            .and_modify(|v| *v = addr.clone())
                            .or_insert(addr);
                    }
                    PeerEvent::ESTABLISHED { host_id, .. } => {
                        peers.contains_key(&host_id);
                        // 检查peer表中是否存在
                        // 续传或新传输
                    }
                    PeerEvent::TRANSFERRING(_) => {}
                    PeerEvent::UNREACHABLE(_) => {

                        // 这个状态是由EST转TRAN失败或长时间没有续传导致的
                        // 提醒拦截器清理，需要重新握手
                    }
                    _ => panic!("unexpected event received"),
                };
            }
        });
    }

    // 注册下游服务
    pub fn registry(&self, sln: &'static impl PeerEventSolution) {
        info!("事件处理器注册");
        PeerCtrlCode::from_bits(sln.interest())
            .unwrap()
            .iter()
            .for_each(|c| {
                self.solution_mapping
                    .entry(c.bits())
                    .and_modify(|v| v.push(sln));
            });
    }
}
