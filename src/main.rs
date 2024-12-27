#![feature(ip)]
use std::{sync::Arc, time::Duration};

use dashmap::DashMap;
use peer::{cursor::Cursor, future_state::PeerFutureState};
use tokio::{spawn, time::interval};
use uid::Uid;
//记得提权
use crate::negotiate::Discovery;

mod env;
mod negotiate;

mod addr;
mod msg;
mod pair;
mod peer;
mod uid;
#[tokio::main]
async fn main() {
    let d0 = Arc::new(Discovery::new().await.unwrap());
    let d1 = d0.clone();
    let d2 = d0.clone();
    let peers: DashMap<Uid, PeerFutureState> = DashMap::new();
    let ppp = Arc::new(peers);
    let pppp = ppp.clone();
    let mut cursor = Cursor::new(ppp.clone());

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
            it1.tick().await;
            d0.listen(ppp.clone()).await;
        }
    });
    spawn(async move {
        loop {
            it2.tick().await;
            d2.select(cursor.get_next_record()).await;
        }
    });
    loop {
        it3.tick().await;
        println!("{:?}", pppp);
    }
}
