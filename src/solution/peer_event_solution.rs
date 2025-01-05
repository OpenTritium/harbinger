use crate::addr::ipv6_scope::Ipv6Scope;
use crate::env::uid::Uid;
use crate::event::peer_event::PeerEvent;
use dashmap::DashMap;
use futures::future::BoxFuture;
use std::error::Error;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;

pub type CrossError = Box<dyn Error + Send + Sync>;
pub type FutureResult<T> = BoxFuture<'static, Result<T, CrossError>>;
pub type SolutionClosure = dyn FnOnce(Arc<UdpSocket>, Arc<DashMap<Uid, Ipv6Scope>>, Sender<PeerEvent>) -> FutureResult<()>;

// 订阅者接口，在观察者眼中的接口
pub trait PeerEventSolution:Send + Sync {
    // 倘若我通知你感兴趣的事情已经发生了，告诉我怎样做
    // 要求签名传入表和sock
    fn dispatch_solution(
        &self,
        event: Box<PeerEvent>,
    ) -> Result<Box<SolutionClosure>, CrossError>;

    // 你感兴趣什么，你还可以指定下一步触发的事件
    fn interest(&self) -> u8;
}