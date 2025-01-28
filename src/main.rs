#![feature(ip)]
#![feature(str_as_str)]
#![feature(unboxed_closures)]
#![feature(duration_constructors)]
extern crate core;

use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

mod addr_v6;
mod utils;
mod msg;
mod peer;

#[tokio::main]
async fn main() {

}
