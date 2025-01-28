use crate::utils::uid::Uid;
use std::net::Ipv6Addr;
use std::str::FromStr;
use std::sync::OnceLock;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct Env {
    pub host_id: RwLock<Uid>,
    pub multicast_local: Ipv6Addr,
    pub multicast_global: Ipv6Addr,
    pub protocol_port: u16,
    pub protocol_version: u8,
}

static ENV: OnceLock<Env> = OnceLock::new();

impl Env {
    pub fn instance() -> &'static Env {
        ENV.get_or_init(|| Env {
            host_id: RwLock::new(nanoid::nanoid!().into()),
            multicast_local: Ipv6Addr::from_str("FF12::1").unwrap(),
            multicast_global: Ipv6Addr::from_str("FF1E::1").unwrap(),
            protocol_port: 5555,
            protocol_version: 0x0,
        })
    }
    pub async fn regen_host_id(&self) {
        *self.host_id.write().await = nanoid::nanoid!().into();
    }
}
