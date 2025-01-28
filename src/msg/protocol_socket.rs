use crate::addr_v6::{Ipv6Scope, ScopeId, ScopeWithPort};
use crate::msg::msg::Msg;
use crate::msg::msg_codec::MsgCodec;
use crate::utils::{Env, NetworkInterfaceView};
use anyhow::Error as AnyError;
use anyhow::Result;
use futures::SinkExt;
use futures::StreamExt;
use futures::stream::{SplitSink, SplitStream};
use std::net::{SocketAddr, SocketAddrV6};
use std::ops::Deref;
use tokio::sync::mpsc::{Receiver, Sender, channel};
use tokio_util::udp::UdpFramed;

pub type MsgSink = SplitSink<UdpFramed<MsgCodec>, (Msg, SocketAddr)>;
pub type MsgStream = SplitStream<UdpFramed<MsgCodec>>;
pub type Parcel = (Msg, Ipv6Scope);
pub type ParcelSender = Sender<Parcel>;
pub type ParcelReceiver = Receiver<Parcel>;

pub struct ProtocolSocket;

// 响应网络切换事件
impl ProtocolSocket {
    pub async fn init() -> Result<(MsgSink, MsgStream)> {
        // todo 内外网支持,clone优化
        let ifaces = NetworkInterfaceView::instance().get().await;
        let (lan, wan) = ifaces.deref();
        let iface_linklocal: Ipv6Scope = lan
            .clone()
            .ok_or(AnyError::msg("linklocal iface not found"))?
            .try_into()?;
        let sock_addr: SocketAddrV6 =
            ScopeWithPort::new_outbound(iface_linklocal, Env::instance().protocol_port)
                .await
                .into();
        let scope_id: ScopeId = sock_addr.scope_id().into();
        let sock = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let sock = tokio::net::UdpSocket::bind(sock_addr).await.expect("failed to bind udpsocket");
                // sock.join_multicast_v6(&Env::instance().multicast_global, 0).unwrap();
                sock
            })
        });
        sock.join_multicast_v6(&Env::instance().multicast_local, scope_id.into())
            .unwrap();
        Ok(UdpFramed::new(sock, MsgCodec::default()).split())
    }

    pub fn sending(mut sink: MsgSink) -> ParcelSender {
        let (tx, mut rx) = channel(128);
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    loop {
                        let (msg, dest) = rx.recv().await.expect("parcel -> sink was broken");
                        let dest: SocketAddrV6 =
                            ScopeWithPort::new(dest, Env::instance().protocol_port).into();
                        sink.send((msg, dest.into())).await.unwrap();
                    }
                });
        });
        tx
    }

    pub fn receiving(mut stream: MsgStream) -> ParcelReceiver {
        let (tx, rx) = channel(128);
        tokio::task::spawn_blocking(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    loop {
                        while let Ok((msg, src)) = stream.next().await.unwrap() {
                            if let SocketAddr::V6(src) = src {
                                if let Ok(src) = Ipv6Scope::try_from(&src) {
                                    tx.send((msg, src)).await.unwrap();
                                } else {
                                    // todo 日志
                                }
                            }
                        }
                    }
                })
        });
        rx
    }
}
