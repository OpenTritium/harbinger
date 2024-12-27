#![feature(ip)]
use std::sync::Arc;

use dashmap::DashMap;
use peer::{cursor::Cursor, future_state::PeerFutureState};
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
    let x = Discovery::new().await.unwrap();

    let peers: DashMap<Uid, PeerFutureState> = DashMap::new();

    let peers = Arc::new(peers);
    let mut cursor = Cursor::new(peers.clone());

    loop {
        x.hello().await;
        x.listen(peers.clone()).await;
       x.select(cursor.get_next_record()).await;
        println!("{:?}",peers);
    }
}
