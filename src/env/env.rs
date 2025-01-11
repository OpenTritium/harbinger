use crate::addr_v6::scope::Ipv6Scope;
use crate::addr_v6::scope::Ipv6Scope::LinkLocal;
use crate::env::uid::Uid;
use nanoid::nanoid;
use rustc_hash::FxHashMap;
use std::net::{IpAddr, Ipv6Addr};
use std::sync::OnceLock;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EnvError {
    #[error("")]
    BestMetricNicNotFound,
    #[error("")]
    LocalLinkNotFound,
    #[error("")]
    RouteUnavailable,
}

#[derive(Debug)]
pub struct Env {
    pub host_id: Uid,
    // scope_id,metric,addr_v6
    pub nics: FxHashMap<u32, (u32, Vec<Ipv6Scope>)>,
    // todo! 从配置读取
    pub multicast_local: Ipv6Addr,
    pub multicast_global: Ipv6Addr,
    pub port: u16,
}

impl Env {
    pub fn best_local_link(&self) -> Result<Ipv6Scope, EnvError> {
        if self.nics.is_empty() {
            return Err(EnvError::RouteUnavailable);
        }
        // pick the best nic by metric
        let (_, (_, addrs)) = self.nics.iter().min_by_key(|r| r.0).unwrap();
        addrs
            .iter()
            .rev()
            .find_map(|addr| match addr {
                a @ LinkLocal { .. } => Some(a.clone()),
                _ => None,
            })
            .ok_or(EnvError::BestMetricNicNotFound)
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
                                Some((a, adapter.ipv6_if_index().into()).try_into().ok()?)
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
        multicast_global: Ipv6Addr::new(0b1111111100011110, 0, 0, 0, 0, 0, 0, 1).into(),
        port: 5555,
    })
}
