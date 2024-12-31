use crate::env::uid::Uid;
use crate::peer::peer_state::PeerState;
use crate::peer::peer_state_code::PeerStateCode;
use dashmap::DashMap;
use std::sync::Arc;

struct Anchor {
    peers: DashMap<Uid, PeerState>,
}

impl Anchor {
    fn new() -> Self {
        Self { peers: DashMap::new() }
    }
    
    // 注册下游服务
    fn registry(handler: &impl Crew) {}
    fn append_peer(&mut self, peer: PeerState) {}
}


// 订阅者接口，在观察者眼中的接口
trait Crew {
    // 倘若我通知你感兴趣的事情已经发生了，告诉我怎样做
    //要求签名传入表
    fn on_status(status: Arc<PeerState>) -> Arc<dyn Fn() + Send + 'static>;
    // 你感兴趣什么
    fn subscribe() -> PeerStateCode;
}