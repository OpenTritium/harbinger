use crate::msg::Msg;
use  crate::peer::{PeerEvent, PeerEventFlags};
use  crate::utils::Env;
use std::net::SocketAddrV6;
use crate::addr_v6::ScopeWithPort;
use std::sync::OnceLock;
use anyhow::Error as AnyError;

static HELLO_REPLY_INSTANCE: OnceLock<HelloReply> = OnceLock::new();

#[derive(Default)]
pub struct HelloReply;

impl crate::peer::PeerEventSolution for HelloReply {
    fn dispatch_solution(&self, event: Box<PeerEvent>) -> crate::peer::DispatchResult {
        if let PeerEvent::Hello { host_id, addr } = *event {
            Ok(Box::new(
                move |parcel_sender, peer_state_table, event_loopback| {
                    Box::pin(async move {
                        let local_host_id = (*Env::instance().host_id.read().await).clone();
                        if host_id == local_host_id {
                            parcel_sender.send((Msg::Conflict, addr)).await?;
                            // log
                            return Ok(());
                        }
                        let dest:SocketAddrV6 = ScopeWithPort::new_outbound(addr, Env::instance().protocol_port).await.try_into()?;
                        parcel_sender.send((
                            Msg::Connect {
                                host_id: local_host_id,
                            },
                            (&dest).try_into()?,
                        )).await?;
                        event_loopback.send(PeerEvent::Established).await.unwrap();
                        // todo 总是更新状态表
                        // todo 考虑解构任务与会话
                        peer_state_table
                            .entry(host_id)
                            .and_modify(|v| *v = addr)
                            .or_insert(addr);
                        Ok(())
                    })
                },
            ))
        } else {
            Err(AnyError::msg("hello replay solution can not handle this event"))
        }
    }

    fn interest(&self) -> &'static PeerEventFlags {
        &Self::INTEREST
    }
}

impl HelloReply {
    const INTEREST: PeerEventFlags = PeerEventFlags::HELLO;
    pub fn instance() -> &'static HelloReply {
        HELLO_REPLY_INSTANCE.get_or_init(HelloReply::default)
    }
}
