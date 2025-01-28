use super::peer_event::PeerEvent;
use super::peer_event_flags::PeerEventFlags;
use super::peer_event_solution::PeerEventSolution;
use crate::addr_v6::Ipv6Scope;
use crate::msg::{ParcelSender,EventReceiver, EventSender};
use crate::utils::{Env,Uid};
use dashmap::DashMap;
use std::sync::Arc;

pub type PeerStateTable = Arc<DashMap<Uid, Ipv6Scope>>;
pub type SolutinMapping = Arc<DashMap<u8, Vec<&'static dyn PeerEventSolution>>>;
pub struct PeerEventHandler {
    pub peer_states: PeerStateTable,
    solution_mapping: SolutinMapping,
}

impl Default for PeerEventHandler {
    fn default() -> Self {
        Self {
            peer_states: Default::default(),
            solution_mapping: [
                (PeerEventFlags::HELLO.bits(), vec![]),
                (PeerEventFlags::CONNECTED.bits(), vec![]),
            ]
            .into_iter()
            .collect::<DashMap<u8, Vec<&'static dyn PeerEventSolution>>>()
            .into(),
        }
    }
}

// service manager
impl PeerEventHandler {
    pub fn handling(
        &self,
        mut event_receiver: EventReceiver,
        parcel_sender: ParcelSender,
        event_loopback: EventSender,
    ) {
        let peer_states = self.peer_states.clone();
        let solution_mapping = self.solution_mapping.clone();
        tokio::spawn(async move {
            // todo 实现 peerevent into 到 flags
            match event_receiver.recv().await.unwrap() {
                // todo 考虑改map
                hello @ PeerEvent::Hello { .. } => {
                    if let Some(slns) = solution_mapping.get(&PeerEventFlags::HELLO.bits()) {
                        for sln in slns.iter() {
                            sln.dispatch_solution(hello.clone().into()).unwrap()(
                                parcel_sender.clone(),
                                peer_states.clone(),
                                event_loopback.clone(),
                            )
                            .await
                            .unwrap();
                        }
                    }
                }

                connected @ PeerEvent::Connect { .. } => {
                    if let Some(slns) = solution_mapping.get(&PeerEventFlags::CONNECTED.bits()) {
                        for sln in slns.iter() {
                            sln.dispatch_solution(connected.clone().into()).unwrap()(
                                parcel_sender.clone(),
                                peer_states.clone(),
                                event_loopback.clone(),
                            )
                            .await
                            .unwrap();
                        }
                    }
                }
                PeerEvent::Established => todo!(),
                PeerEvent::Conflict { addr } => {
                    // 过滤来自广播的冲突信息
                    Env::instance().regen_host_id().await;
                }
            }
        });
    }

    pub fn registry(&self, sln: &'static impl PeerEventSolution) {
        sln.interest().iter().for_each(|flag| {
            self.solution_mapping
                .entry(flag.bits())
                .and_modify(|v| v.push(sln));
        });
    }
}
