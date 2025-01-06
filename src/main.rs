#![feature(ip)]
#![feature(str_as_str)]
#![feature(unboxed_closures)]
#![feature(duration_constructors)]
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
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
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    repeating_hello();
    peer_event_executor().registry(hello_reply());
    peer_event_executor().listening().await;
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
        loop {
            interval.tick().await;
            info!("已发现的用户：{:?}",peer_event_executor().peers);
            //tokio::task::yield_now().await;
        }
    });
    loop {
        std::thread::park();
    }
}
