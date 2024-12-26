use nanoid::nanoid;
use rustc_hash::FxHashMap;
use std::sync::OnceLock;
use crate::addr::addr::Ipv6Scope;
use crate::uid::Uid;
use std::net::{IpAddr, Ipv6Addr};

#[derive(Debug)]
pub struct Env {
    pub host_id: Uid,
    pub nics: FxHashMap<u32, (u32, Vec<Ipv6Scope>)>,
    // todo! 从配置读取
    pub multicast_local: Ipv6Addr,
    pub multicast_global: Ipv6Addr,
    pub port: u16,
}

impl Env {
    pub fn is_ipv6_route_available(&self) -> bool {
        !self.nics.is_empty()
    }
}

static ENV: OnceLock<Env> = OnceLock::new();

pub fn get_env() -> &'static Env {
    ENV.get_or_init(|| Env {
        host_id: nanoid!().into(),
        nics: ipconfig::get_adapters()
            .unwrap()
            .into_iter()
            .filter_map(|adapter| {
                if !adapter.gateways().iter().any(|ip| ip.is_ipv6()) {
                    None
                } else {
                    let ipv6_addrs = adapter
                        .ip_addresses()
                        .iter()
                        .filter_map(|&ip| {
                            if let IpAddr::V6(a) = ip {
                                Some(
                                    Ipv6Scope::try_from_ipv6addr(
                                        &a,
                                        Some(adapter.ipv6_if_index().into()),
                                    )
                                    .ok()?,
                                )
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    return if !ipv6_addrs.is_empty() {
                        Some((adapter.ipv6_if_index(), (adapter.ipv6_metric(), ipv6_addrs)))
                    } else {
                        None
                    };
                }
            })
            .collect::<FxHashMap<_, _>>(),
        multicast_local: Ipv6Addr::new(0b1111111100010010, 0, 0, 0, 0, 0, 0, 1),
        multicast_global: Ipv6Addr::new(0b1111111100011110, 0, 0, 0, 0, 0, 0, 1),
        port: 5555,
    })
}
