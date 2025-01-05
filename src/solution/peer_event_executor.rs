use crate::addr::ipv6_scope::Ipv6Scope;
use crate::env::uid::Uid;
use crate::event::peer_event::PeerEvent;
use crate::protocol::peer_ctrl_code::PeerCtrlCode;
use crate::solution::peer_event_solution::PeerEventSolution;
use dashmap::DashMap;
use std::sync::{Arc, OnceLock};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use crate::interceptor::msg_interceptor::msg_interceptor;
use crate::msg::msg_socket::msg_socket;

pub static PEER_EVENT_EXECUTOR_INSTANCE: OnceLock<PeerEventExecutor> = OnceLock::new();
pub fn peer_event_executor() -> &'static PeerEventExecutor {
    PEER_EVENT_EXECUTOR_INSTANCE.get_or_init(PeerEventExecutor::new)
}
pub struct PeerEventExecutor {
    pub peers: Arc<DashMap<Uid, Ipv6Scope>>,
    solution_mapping: Arc<DashMap<u8, Vec<&'static dyn PeerEventSolution>>>,
    socket:Arc<UdpSocket>
}

impl PeerEventExecutor {
    pub fn new() -> Self {
        println!("{}","事件执行器初始化");
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
            socket: msg_socket().get_raw_socket()
        }
    }
    // 上游消息流
    // 多播只会收到hello
    //单播收到opt

    pub fn listening(&self) ->Sender<PeerEvent> {
        let (loopback, mut listener) = channel::<PeerEvent>(128);
        let loopback_cloned = loopback.clone();
        let peers = self.peers.clone();
        let solution_mapping = self.solution_mapping.clone();
        let socket = self.socket.clone();
        tokio::spawn(async move {
            // 消费一定要跟上速度，不然通道会被挤爆的
            loop {
            let event = listener.recv().await.unwrap();
            match event {
                // 收到 hello 准备回复
                e @ PeerEvent::HELLO(..) => {
                    println!("触发hello事件处理");
                    if let Some(v) = solution_mapping.get(&PeerCtrlCode::HELLO.bits()) {
                        for s in v.iter() {
                            let closure = s.dispatch_solution(e.clone().into()).unwrap();
                            closure(socket.clone(), peers.clone(), loopback.clone()).await.unwrap();
                        }
                    }
                }
                // 如果表中有此用户，不插入，如果没有就插入
                PeerEvent::CONNECTED(uid, addr) => {
                    println!("触发连接");
                }
                // PeerEvent::ESTABLISHED(uid) => {
                //
                //     self.peers.contains_key(&uid);
                //     // 检查peer表中是否存在
                //     // 续传或新传输
                // }
                PeerEvent::TRANSFERRING(_) => {}
                // PeerEvent::UNREACHABLE(_) => {
                //
                //     // 这个状态是由EST转TRAN失败或长时间没有续传导致的
                //     // 提醒拦截器清理，需要重新握手
                // }
                _ => panic!("unexpected event received"),
            };
        }
    });
    loopback_cloned
    }

    // 注册下游服务
    pub fn registry(&self, sln: &'static impl PeerEventSolution) {
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
