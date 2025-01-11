use crate::env::env::get_env;
use crate::event::peer_event::PeerEvent;
use crate::msg::ctrl_msg::CtrlMsg;
use crate::msg::msg::Msg;
use crate::protocol::peer_ctrl_code::PeerCtrlCode;
use crate::solution::peer_event_solution::{
    CrossError, FutureResult, PeerEventSolution, SolutionClosure,
};
use std::net::{SocketAddr, SocketAddrV6};
use std::sync::OnceLock;
use thiserror::Error;
use tracing::info;
use crate::addr_v6::scope::ScopeWithPort;

static CONNECTION_REQUEST: OnceLock<Msg> = OnceLock::new();
static HELLO_REPLY_INSTANCE: OnceLock<HelloReply> = OnceLock::new();
pub fn hello_reply() -> &'static HelloReply {
    info!("{}", "回复事件处理器初始化");
    HELLO_REPLY_INSTANCE.get_or_init(HelloReply::new)
}
pub struct HelloReply {
    interest: u8,
}
#[derive(Debug, Error)]
pub enum HelloReplyError {
    #[error("")]
    EventNotMatch,
}

impl PeerEventSolution for HelloReply {
    fn dispatch_solution(&self, event: Box<PeerEvent>) -> Result<Box<SolutionClosure>, CrossError> {
        if let PeerEvent::HELLO{host_id, addr } = *event {
            info!("HELLO received!");
            let msg = CONNECTION_REQUEST
                .get_or_init(|| {
                    Msg::Ctrl(CtrlMsg::new(
                        PeerCtrlCode::CONNECT.bits(),
                        get_env().host_id.clone(),
                        get_env().best_local_link().unwrap(),
                    ))
                })
                .clone();
            info!("构造回复消息：{}", msg);
            Ok(Box::new(
                move |sender, peers, loopback| -> FutureResult<()> {
                    Box::pin(async move {
                        let dest:SocketAddrV6 = ScopeWithPort{scope:addr.clone().replace_scope_id().unwrap().into(),port:get_env().port}.into();
                        sender
                            .send((
                                msg,
                                dest.into(),
                            ).into())
                            .await
                            .unwrap();
                        info!("发送连接请求");
                        loopback
                            .send(PeerEvent::ESTABLISHED{host_id:host_id.clone(),addr:addr.clone()})
                            .await
                            .unwrap();
                        info!("发送自循环事件");

                        peers
                            .entry(host_id)
                            .and_modify(|v| *v = addr.clone())
                            .or_insert(addr);
                        info!("插入用户表");
                        Ok(())
                    })
                },
            ))
        } else {
            Err(HelloReplyError::EventNotMatch.into())
        }
    }
    // 这个函数会远远不断的产生不同的事件处理器（因为每个闭包捕获的值实际上不同）

    fn interest(&self) -> u8 {
        self.interest
    }
}

impl HelloReply {
    fn new() -> Self {
        HelloReply {
            interest: PeerCtrlCode::HELLO.bits(),
        }
    }
}
