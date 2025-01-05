use crate::addr::ipv6_scope::Ipv6Scope;
use crate::env::env::{get_env, Env};
use crate::msg::hello_msg::HelloMsg;
use crate::msg::msg_socket::msg_socket;
use std::net::SocketAddrV6;

pub fn repeating_hello() {
    tokio::spawn(async {
        let Env {
            port,
            multicast_local,
            ..
        } = get_env();
        println!("{}","hello 服务工作中");
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
        if let Ipv6Scope::LinkLocal(_, sid) = get_env().best_local_link().unwrap() {
            loop {
                interval.tick().await;
                msg_socket()
                    .get_raw_socket()
                    .send_to(
                        HelloMsg::new().to_string().as_bytes(),
                        SocketAddrV6::new(*multicast_local, *port, 0, sid.clone().into()),
                    )
                    .await
                    .unwrap();
            }

        }
    });
}
