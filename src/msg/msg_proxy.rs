use super::protocol_socket::ParcelSender;
use crate::utils::Uid;
use crate::msg::ProtocolSocket;
use crate::peer::PeerEvent;
use anyhow::Result;
use dashmap::DashSet;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender, channel};

pub type EventSender = Sender<PeerEvent>;
pub type EventReceiver = Receiver<PeerEvent>;

#[derive(Debug)]
pub struct MsgProxy {
    filtered_uid: Arc<DashSet<Uid>>,
}

impl MsgProxy {
    pub async fn proxying() -> Result<(EventSender, EventReceiver, ParcelSender)> {
        let (event_sender, event_receiver) = channel(128);
        let event_sender_clone = event_sender.clone();
        let (msg_sink, msg_stream) = ProtocolSocket::init().await?;
        let parcel_sender = ProtocolSocket::sending(msg_sink);
        tokio::spawn(async move {
            let mut msg_reveiver = ProtocolSocket::receiving(msg_stream);
            loop {
                if let Some(parcel) = msg_reveiver.recv().await {
                    event_sender_clone.send(parcel.into()).await.expect("msg -> event was broken");
                }
            }
        });
        Ok((event_sender, event_receiver, parcel_sender))
    }
    // 实现过滤列表
}
