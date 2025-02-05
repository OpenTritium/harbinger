use std::net::Ipv6Addr;
use std::str::FromStr;
use std::sync::OnceLock;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct Env {
    pub host_id: RwLock<super::Uid>,
    pub multicast_lan: Ipv6Addr,
    pub multicast_wan: Ipv6Addr,
    pub protocol_port: u16,
    pub protocol_version: u8,
}

static ENV: OnceLock<Env> = OnceLock::new();

pub fn env() -> &'static Env {
    ENV.get_or_init(|| Env {
        host_id: RwLock::new(nanoid::nanoid!().into()),
        multicast_lan: Ipv6Addr::from_str("FF12::1").unwrap(),
        multicast_wan: Ipv6Addr::from_str("FF1E::1").unwrap(),
        protocol_port: 5555,
        protocol_version: 0x0,
    })
}

impl Env {
    pub async fn regen_host_id(&self) {
        *self.host_id.write().await = nanoid::nanoid!().into();
    }
}
