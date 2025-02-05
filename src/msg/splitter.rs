use super::ParcelReceiver;
use super::socket::{MsgSink, MsgStream, ProtocolSocketFactory};
use super::{codec::MsgCodec, socket::ParcelSender};
use crate::addr_v6::Ipv6Scope;
use crate::addr_v6::ScopeWithPort;
use crate::peer::PeerEvent;
use crate::utils::{Uid, env};
use Ipv6Scope::*;
use dashmap::DashSet;
use futures::{SinkExt, StreamExt};
use std::net::{SocketAddr, SocketAddrV6};
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio_util::udp::UdpFramed;
use tracing::{info, warn};

pub type EventSender = Sender<PeerEvent>;
pub type EventReceiver = Receiver<PeerEvent>;

#[derive(Debug)]
pub struct MsgSplitter {
    filtered_uid: Arc<DashSet<Uid>>,
}

pub struct ParcelIo {
    pub inbound: ParcelReceiver,
    pub outbound: ParcelSender,
}

impl ParcelIo {
    pub fn new(inbound: ParcelReceiver, outbound: ParcelSender) -> Self {
        Self { inbound, outbound }
    }
}

fn busy_on(f: impl Future + Send + 'static) {
    tokio::task::spawn_blocking(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build runtime")
            .block_on(f);
    });
}

impl MsgSplitter {
    fn forwarding_stream_to_channel(mut stream: MsgStream, stream_parcel_in: ParcelSender) {
        busy_on(async move {
            while let Ok((msg, sock_addr)) = stream
                .next()
                .await
                .expect("The message stream terminated unexpectedly.")
            {
                info!("分流器入站：{} 内容 {:?}", sock_addr, &msg);
                let SocketAddr::V6(sock_addr) = sock_addr else {
                    warn!("non-IPv6 addr: {}.", sock_addr);
                    continue;
                };
                let Ok(addr) = (&sock_addr).try_into() else {
                    warn!(
                        "Address cannot be converted by Ipv6Scope: non-convertible or invalid IPv6 address: {:?}.",
                        sock_addr
                    );
                    continue;
                };
                stream_parcel_in
                    .send((msg, addr))
                    .await
                    .expect("Failed to send messages from stream to parcel channel.");
            }
            warn!("流中断");
            panic!(); //todo
        });
    }

    fn forwarding_channel_to_sink(
        sinks: Box<[Option<MsgSink>; 2]>,
        mut sink_parcel_out: ParcelReceiver,
    ) {
        busy_on(async move {
            let [mut lan_sink, mut wan_sink] = *sinks;
            loop {
                let (msg, dest) = sink_parcel_out.recv().await.expect(
                    "The parcel channel for sink message delivery has closed unexpectedly.",
                );
                info!("SINK 积压：{}",sink_parcel_out.len());
                info!("分流器出站：{} 内容：{:?} ", dest, &msg);
                match dest {
                    Lan { .. } if lan_sink.is_some() => {
                        let sock_addr: SocketAddrV6 =
                            ScopeWithPort::new(dest, env().protocol_port).into();
                        lan_sink
                            .as_mut()
                            .unwrap()
                            .send((msg, sock_addr.into()))
                            .await
                            .unwrap();
                    }
                    Wan(_) if wan_sink.is_some() => {
                        let sock_addr: SocketAddrV6 =
                            ScopeWithPort::new(dest, env().protocol_port).into();
                        wan_sink
                            .as_mut()
                            .unwrap()
                            .send((msg, sock_addr.into()))
                            .await
                            .unwrap();
                    }
                    _ => {
                        warn!(
                            "Dest: {:?}; No suitable socket type available for the message: {:?}.",
                            dest, msg
                        );
                        continue;
                    }
                };
            }
        });
    }
    // newtype for rt val
    pub async fn forwarding() -> ParcelIo {
        let (sinks, mut streams): (Vec<_>, Vec<_>) = ProtocolSocketFactory::new()
            .await
            .into_iter()
            .map(|sock| {
                sock.inspect_err(|err| warn!("{}", err))
                    .ok()
                    .map(|sock| UdpFramed::new(sock, MsgCodec).split())
            })
            .map(|opt| {
                opt.map(|(sink, stream)| (Some(sink), Some(stream)))
                    .unwrap_or((None, None))
            })
            .unzip();
        let (stream_parcel_in, stream_parcel_out) = channel(128);
        let (sink_parcel_in, sink_parcel_out) = channel(128);
        (0..2).for_each(|n| {
            if let Some(stream) = streams[n].take() {
                Self::forwarding_stream_to_channel(stream, stream_parcel_in.clone());
            }
        });
        Self::forwarding_channel_to_sink(
            sinks.into_boxed_slice().try_into().unwrap(),
            sink_parcel_out,
        );
        ParcelIo::new(stream_parcel_out, sink_parcel_in)
    }
}
