use nanoid::nanoid;
use rustc_hash::FxHashMap;
use std::sync::OnceLock;

use std::net::{IpAddr, Ipv6Addr};

#[derive(Debug)]
enum Ipv6Scope {
    LinkLocal(Ipv6Addr),
    Global(Ipv6Addr),
}

#[derive(Debug)]
pub struct Env {
    host_id: String,
    nics: FxHashMap<u32, Vec<Ipv6Scope>>,
}

static ENV: OnceLock<Env> = OnceLock::new();

pub fn get_env() -> &'static Env {
    ENV.get_or_init(|| Env {
        host_id: nanoid!(),
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
                        .filter_map(|&ip| match ip {
                            IpAddr::V6(ip) => Some(if ip.is_global() {
                                Ipv6Scope::Global(ip)
                            } else if ip.is_unicast_link_local() {
                                Ipv6Scope::LinkLocal(ip)
                            } else {
                                return None;
                            }),
                            _ => None,
                        })
                        .collect::<Vec<_>>();
                    return if !ipv6_addrs.is_empty() {
                        Some((adapter.ipv6_if_index(), ipv6_addrs))
                    } else {
                        None
                    };
                }
            })
            .collect::<FxHashMap<_, _>>(),
    })
}
