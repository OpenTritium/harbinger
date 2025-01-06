use crate::env::env::get_env;
use crate::event::peer_event::PeerEvent;
use crate::msg::msg::Message;
use crate::msg::msg_socket::msg_socket;
use crate::solution::peer_event_executor::peer_event_executor;
use std::sync::OnceLock;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_stream::StreamExt;
use tracing::{info, instrument};

static MSG_INTERCEPTOR_INSTANCE: OnceLock<MsgInterceptor> = OnceLock::new();
pub fn msg_interceptor() -> &'static MsgInterceptor {
    MSG_INTERCEPTOR_INSTANCE.get_or_init(|| MsgInterceptor::new())
}
#[derive(Debug)]
pub struct MsgInterceptor {}
// 过滤已连接用户的请求连接组播hello报文
impl MsgInterceptor {
    pub async fn bridge_and_filtering(&self) -> (Sender<PeerEvent>, Receiver<PeerEvent>) {
        // 过滤 msg
        let (tx, rx) = channel(128);
        let downstream = tx.clone();
        let mut rs = msg_socket().msg_streaming().await;
        info!("桥接 socket 消息流 -> PeerEventExecutor");
        tokio::spawn(async move {
            loop {
                if let Some(msg) = rs.recv().await {
                    if match &msg {
                        Message::Hello(hello) => hello.host_id != get_env().host_id,
                        Message::Ctrl(ctrl) => ctrl.host_id != get_env().host_id,
                    } {
                        downstream.send(msg.into()).await.unwrap();
                        info!("将 socket 消息 转发到 PeerEventExecutor");
                    }
                }
            }
        });
        (tx, rx)
    }
    pub fn new() -> Self {
        info!("{}", "消息拦截器初始化");
        Self {}
    }
}
