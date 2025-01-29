#![feature(ip)]
#![feature(str_as_str)]
#![feature(unboxed_closures)]
#![feature(duration_constructors)]

use std::{thread::{park, sleep}, time::Duration};

use msg::{MsgProxy, ProtocolSocket};
use peer::{hello_reply::HelloReply, repeating_hello, PeerEventHandler};
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

mod addr_v6;
mod utils;
mod msg;
mod peer;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::fmt().init();
    let (event_sender,event_receiver,parcel_sender) = MsgProxy::proxying().await.unwrap();
    let peh = PeerEventHandler::default();
    peh.handling(event_receiver, parcel_sender.clone(), event_sender);
    peh.registry(HelloReply::instance());
    repeating_hello(parcel_sender);
    sleep(Duration::from_days(1));
}
