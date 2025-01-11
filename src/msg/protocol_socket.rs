use crate::addr_v6::scope::Ipv6Scope::LinkLocal;
use crate::addr_v6::scope::ScopeWithPort;
use crate::env::env::{get_env, EnvError};
use crate::msg::msg::Msg;
use crate::msg::msg_codec::MsgCodec;
use futures::stream::{SplitSink, SplitStream};
use futures::SinkExt;
use futures::StreamExt;
use std::net::{SocketAddr, SocketAddrV6};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::task::block_in_place;
use tokio_util::udp::UdpFramed;

pub struct ProtocolSocket {
    link_local_sink: Arc<Mutex<SplitSink<UdpFramed<MsgCodec>, (Msg, SocketAddr)>>>,
    link_local_stream: Arc<Mutex<SplitStream<UdpFramed<MsgCodec>>>>,
}

static MSG_SOCKET_INSTANCE: OnceLock<ProtocolSocket> = OnceLock::new();
pub fn protocol_socket() -> &'static ProtocolSocket {
    MSG_SOCKET_INSTANCE.get_or_init(|| ProtocolSocket::new().unwrap())
}

static PROTOCOL_SOCKET_SENDER_COPY: OnceLock<Sender<(Msg, SocketAddr)>> = OnceLock::new();

impl ProtocolSocket {
    pub fn new() -> Result<Self, EnvError> {
        let bll = get_env().best_local_link()?;
        if let LinkLocal { scope_id, .. } = bll {
            let sock_addr: SocketAddrV6 = ScopeWithPort {
                scope: bll,
                port: get_env().port,
            }
            .into();
            let sock = block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async move {
                    let sock = UdpSocket::bind(sock_addr).await.unwrap();
                    sock.join_multicast_v6(&get_env().multicast_local, scope_id.into())
                        .unwrap();
                    sock
                })
            });
            let (sink, stream) = UdpFramed::new(sock, MsgCodec::new()).split();
            Ok(Self {
                link_local_stream: Arc::new(Mutex::new(stream)),
                link_local_sink: Arc::new(Mutex::new(sink)),
            })
        } else {
            Err(EnvError::BestMetricNicNotFound)
        }
    }
    //todo once 不要重复调用此函数
    pub fn sending(&self) -> Sender<(Msg, SocketAddr)> {
        let (tx, mut rx) = channel(128);
        let sender = self.link_local_sink.clone();
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("无法创建运行时"); // todo
            rt.block_on(async move {
                let mut sg = sender.lock().unwrap();
                loop {
                    let (msg, dest) = rx.recv().await.unwrap();
                    sg.send((msg, dest)).await.unwrap();
                }
            });
        });
        tx
    }
    //todo once 不要重复调用此函数
    pub fn receiving(&self) -> Receiver<Msg> {
        let receiver = self.link_local_stream.clone();
        let (tx, rx) = channel(128);
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                let mut r = receiver.lock().unwrap();
                loop {
                    while let Ok((msg, _)) = r.next().await.unwrap() {
                        tx.send(msg).await.unwrap();
                    }
                }
            })
        });
        rx
    }
    pub fn get_sender() -> Sender<(Msg, SocketAddr)> {
        PROTOCOL_SOCKET_SENDER_COPY
            .get_or_init(|| protocol_socket().sending())
            .clone()
    }
}
