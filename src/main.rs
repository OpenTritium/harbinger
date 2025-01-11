#![feature(ip)]
#![feature(str_as_str)]
#![feature(unboxed_closures)]
#![feature(duration_constructors)]
extern crate core;

use crate::service::repeat_hello::repeating_hello;
use crate::solution::hello_reply::hello_reply;
use crate::solution::peer_event_handler::peer_event_handler;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

mod protocol;

mod addr_v6;
mod env;
mod event;
mod interceptor;
mod msg;
mod service;
mod solution;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    repeating_hello();
    peer_event_handler().registry(hello_reply());
    peer_event_handler().listening().await;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
        loop {
            interval.tick().await;
            info!("已发现的用户：{:?}", peer_event_handler().peers);
            //tokio::task::yield_now().await;
        }
    });
    loop {
        std::thread::park();
    }
}
