#![feature(ip)]
use std::{sync::Arc, time::Duration};

use crate::env::uid::Uid;
//记得提权
use crate::discovery::Discovery;
use dashmap::DashMap;
use peer::{peer_state::PeerState};
use tokio::{spawn, time::interval};

mod discovery;

mod addr;
mod msg;
mod peer;
mod env;

#[tokio::main]
async fn main() {
    let d0 = Arc::new(Discovery::new().await.unwrap());
    let d1 = d0.clone();
    let d2 = d0.clone();
    let peers: DashMap<Uid, PeerState> = DashMap::new();
    let ppp = Arc::new(peers);
    let pppp = ppp.clone();

    let mut it = interval(Duration::from_secs(3));
    let mut it1 = interval(Duration::from_secs(3));
    let mut it2 = interval(Duration::from_secs(3));
    let mut it3 = interval(Duration::from_secs(3));
    spawn(async move {
        loop {
            it.tick().await;
            d1.hello().await;
        }
    });
    spawn(async move {
        loop {
            d0.listen(ppp.clone()).await;
        }
    });
    loop {
        it3.tick().await;
        println!("{:?}", pppp);
    }
}
