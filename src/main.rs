#![feature(ip)]
#![feature(inherent_associated_types)]

use crate::negotiate::Discovery;

mod env;
mod negotiate;

mod msg;
mod pair;
mod peer;
mod addr;
mod uid;

#[tokio::main]
async fn main() {
    let x = Discovery::new().await.unwrap();
    x.hello().await;
    let xx = x.on_discovering().await;
}
