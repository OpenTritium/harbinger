#![feature(ip)]
#![feature(str_as_str)]
#![feature(unboxed_closures)]
#![feature(duration_constructors)]

use std::thread::park;

use msg::{MsgEventAdapter, MsgSplitter};
use peer::{repeating_hello, PeerEventHandler};



mod addr_v6;
mod msg;
mod peer;
mod utils;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt().init();
    //console_subscriber::init();
    // let lan = netif::up().unwrap().find(|iface| {
    //     // Add your condition here, for example:
    //     iface.name().to_lowercase().contains("eth") && iface.is_ipv6()
    // }).unwrap();
    let x  = MsgSplitter::forwarding().await;
    let y = MsgEventAdapter::accpeting(x);
    let xx = y.parcel_sender.clone();
    let h = PeerEventHandler::default();
    h.handling(y).await;
    repeating_hello(xx).await.unwrap();
    park();
}
