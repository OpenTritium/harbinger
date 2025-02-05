use super::flags::PeerEventFlags;
use crate::msg::{AdapterIo, Msg};
use crate::peer::{PeerEvent, event::StateEntry};
use crate::utils::env;
use PeerEvent::*;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Default)]
pub struct PeerEventHandler {
    pub peers_state: Arc<super::state_table::StateTable>,
}

// service manager
impl PeerEventHandler {
    pub async fn handling(&self, adapter_io: crate::msg::AdapterIo) {
        let AdapterIo {
            loopback,
            mut receiver,
            parcel_sender,
        } = adapter_io;
        let peers_state = self.peers_state.clone();

        tokio::spawn(async move {
            // 不要关心网卡在不在的问题，这是是下面一层该关心的问题
            loop {
                let event = receiver.recv().await.unwrap();
                info!("开始处理事件：{:?}",event);
                match event {
                    //todo 过滤组播hello
                    Hello {
                        host_id: remote_host_id,
                        addr: dest, // 下层已经将组播地址换成真实目标地址了
                    } => {
                        let local_host_id = (*env().host_id.read().await).clone();
                        if local_host_id == remote_host_id {
                            warn!("对方 UID 与本机重复，请求对方变更");
                            parcel_sender.send((Msg::Conflict, dest)).await.unwrap();
                            continue;
                        }
                        let msg = Msg::Connect {
                            host_id: local_host_id,
                        };
                        parcel_sender.send((msg, dest)).await.unwrap();
                        loopback.send(Established).await.unwrap();
                        // 总是无条件插入，上游会基于Hash(uid,addr)拦截已经配对的会话，如果通过拦截，说明uid或addr至少有一个不一样
                        peers_state.insert(
                            remote_host_id,
                            StateEntry::new(dest, PeerEventFlags::ESTABLISHED),
                        );
                    }
                    Connect {
                        host_id: remote_host_id,
                        addr: dest,
                    } => {
                        loopback.send(Established).await.unwrap();
                        peers_state.insert(
                            remote_host_id,
                            StateEntry::new(dest, PeerEventFlags::ESTABLISHED),
                        );
                    }
                    Established => {
                        info!("Pair success");
                    }
                    Conflict { .. } => {
                        warn!("有人提醒我 UID 冲突了，立马修改");
                        env().regen_host_id().await;
                    }
                }
            }
        });
    }
}
