use crate::addr::ipv6_scope::Ipv6Scope::LinkLocal;
use crate::env::env::{get_env, EnvError};
use crate::msg::msg::Message;
use bytes::BytesMut;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddrV6;
use std::str::FromStr;
use std::sync::{Arc, OnceLock};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

#[derive(Clone)]
pub struct MsgSocket {
    link_local_sockaddr: Arc<SocketAddrV6>,
    link_local_socket: Arc<UdpSocket>,
}

static MSG_SOCKET_INSTANCE:OnceLock<MsgSocket> = OnceLock::new();
pub fn msg_socket() -> &'static MsgSocket {
    MSG_SOCKET_INSTANCE.get_or_init(|| MsgSocket::new().unwrap())
}

impl MsgSocket {
    pub fn new() -> Result<Self, EnvError> {
        let bll = get_env().best_local_link()?;
        let sock_addr = bll.clone().into_sockaddr_v6();
        // todo 去除 socket 2
        let s = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP)).unwrap();
        s.set_reuse_address(true).unwrap();
        s.bind(&sock_addr.clone().into()).unwrap();
        if let LinkLocal(_, sid) = bll {
            //todo 错误处理
            s.join_multicast_v6(&get_env().multicast_local, sid.into())
                .unwrap();
        }
        println!("{}","消息源初始化");
        // todo 处理全球
        Ok(Self {
            link_local_sockaddr: Arc::new(sock_addr),
            link_local_socket: Arc::new(UdpSocket::from_std(s.into()).unwrap()),
        })
    }
    // 暂时只允许调用一次
    pub fn msg_streaming(&self) -> UnboundedReceiverStream<Message> {
        let (tx, rx) = mpsc::unbounded_channel::<Message>();
        let lls_copy = self.link_local_socket.clone();
        tokio::spawn(async move {
            let mut buffer = BytesMut::with_capacity(1024);
            loop {
                match lls_copy.recv_buf_from(&mut buffer).await {
                    Ok((len, _)) => {
                        let msg = Message::from_str(
                            String::from_utf8_lossy(&buffer.split_to(len).freeze()).as_str(),
                        )
                        .expect("skip it if it was broken");
                        dbg!(&msg);
                        tx.send(msg).unwrap();
                        println!("已将socket接收数据发送到消息流");
                    }
                    Err(e) => { println!("{:?}", e); }
                }
                buffer.clear();
            }
        });
        UnboundedReceiverStream::new(rx)
    }
    pub fn get_raw_socket(&self) -> Arc<UdpSocket> {
        self.link_local_socket.clone()
    }
}