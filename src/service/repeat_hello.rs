use crate::addr_v6::scope::{Ipv6Scope, ScopeWithPort};
use crate::env::env::{Env, get_env};
use crate::msg::hello_msg::HelloMsg;
use crate::msg::protocol_socket::{protocol_socket, ProtocolSocket};
use std::net::SocketAddrV6;
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;

pub fn repeating_hello() {
    tokio::spawn(async {
        info!("hello 服务运行中");
        let s = ProtocolSocket::get_sender();
        if let Ipv6Scope::LinkLocal {scope_id,..} = get_env().best_local_link().unwrap(){
            let dest:SocketAddrV6 = ScopeWithPort{scope:Ipv6Scope::LinkLocal {addr:get_env().multicast_local, scope_id },port:get_env().port}.into();
            loop{
                sleep(Duration::from_secs(3)).await;
                s.send((HelloMsg::new().into(), dest.into())).await.unwrap();
            }
        }
       
    });
}
