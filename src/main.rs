#![feature(ip)]
#![feature(str_as_str)]
#![feature(unboxed_closures)]
#![feature(duration_constructors)]

use crate::interceptor::msg_interceptor::msg_interceptor;
use crate::service::repeat_hello::repeating_hello;
use crate::solution::hello_reply::hello_reply;
use crate::solution::peer_event_executor::{peer_event_executor};

mod protocol;

mod addr;
mod env;
mod event;
mod interceptor;
mod msg;
mod service;
mod solution;

#[tokio::main]
async fn main() {
    repeating_hello();
    peer_event_executor().registry(hello_reply());
    msg_interceptor().bridge_and_filtering();
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
    loop {
        interval.tick().await;
        println!("{:?}",peer_event_executor().peers);
    }
}
