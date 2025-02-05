use tracing::info;

use super::{EventReceiver, EventSender, ParcelSender, splitter::ParcelIo};

pub struct MsgEventAdapter;

pub struct AdapterIo {
    pub loopback: EventSender,
    pub receiver: EventReceiver,
    pub parcel_sender: ParcelSender,
}

impl AdapterIo {
    fn new(
        event_upstream: EventSender,
        event_downstream: EventReceiver,
        parcel_sender: ParcelSender,
    ) -> Self {
        Self {
            loopback: event_upstream,
            receiver: event_downstream,
            parcel_sender,
        }
    }
}

impl MsgEventAdapter {
    pub fn accpeting(parcel_io: ParcelIo) -> AdapterIo {
        let (event_upstream, event_downstream) = tokio::sync::mpsc::channel(128);
        let ParcelIo {
            mut inbound,
            outbound,
        } = parcel_io;
        let event_upstream_cloned = event_upstream.clone();
        tokio::spawn(async move {
            loop {
                let parcel = inbound.recv().await.expect(
                    "The channel for receiving Parcels in MsgEventAdapter has unexpectedly closed.",
                );
                
                info!("STREAM 积压：{}",inbound.len());
                event_upstream_cloned
                    .send(parcel.into())
                    .await
                    .expect("Failed to send PeerEvent to the PeerEventHandler");
            }
        });
        AdapterIo::new(event_upstream, event_downstream, outbound)
    }
}
