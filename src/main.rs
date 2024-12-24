#![feature(ip)]

mod env;

fn main() {
    let e = env::get_env();
    println!("{:?}", e);
}
