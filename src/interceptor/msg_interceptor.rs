use crate::env::env::get_env;
use crate::env::uid::Uid;
use crate::event::peer_event::PeerEvent;
use crate::msg::msg::Msg;
use crate::msg::protocol_socket::protocol_socket;
use dashmap::DashSet;
use std::sync::{Arc, OnceLock};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tracing::info;

static MSG_INTERCEPTOR_INSTANCE: OnceLock<MsgInterceptor> = OnceLock::new();
pub fn msg_interceptor() -> &'static MsgInterceptor {
    MSG_INTERCEPTOR_INSTANCE.get_or_init(|| MsgInterceptor::new())
}
#[derive(Debug)]
pub struct MsgInterceptor {
    filtered_uid: Arc<DashSet<Uid>>,
}
// 过滤已连接用户的请求连接组播hello报文
impl MsgInterceptor {
    pub async fn bridge_and_filtering(&self) -> (Sender<PeerEvent>, Receiver<PeerEvent>) {
        //todo: 过滤 msg
        let (tx, rx) = channel(128);
        let downstream = tx.clone();
        let mut rxc = protocol_socket().receiving();
        info!("桥接 socket 消息流 -> PeerEventExecutor");
        tokio::spawn(async move {
            loop {
                if let Some(msg) = rxc.recv().await {
                    if match &msg {
                        Msg::Hello(hello) => hello.host_id != get_env().host_id,
                        Msg::Ctrl(ctrl) => ctrl.host_id != get_env().host_id,
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
        Self {
            filtered_uid: Arc::new({
                let s = DashSet::new();
                s.insert(get_env().host_id.clone());
                s
            }),
        }
    }
}
