use super::ScopeId;
use CastMode::*;
use Ipv6Scope::*;
use anyhow::Error as AnyError;
use anyhow::Ok;
use netif::Interface;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv6Addr, SocketAddrV6};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Ipv6Scope {
    Lan { addr: CastMode, scope_id: ScopeId }, // LinkLocal
    Wan(CastMode),                             // scope more wider than LinkLocal
}

impl TryFrom<Ipv6Addr> for CastMode {
    type Error = AnyError;
    fn try_from(val: Ipv6Addr) -> Result<Self, Self::Error> {
        if val.is_unicast() {
            Ok(CastMode::Unicast(val))
        } else if val.is_multicast() {
            Ok(CastMode::Multicast(val))
        } else {
            Err(AnyError::msg(
                "no convertion from ipv6addr to castmode for this value",
            ))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum CastMode {
    Unicast(Ipv6Addr),
    Multicast(Ipv6Addr),
}

impl From<CastMode> for Ipv6Addr {
    fn from(val: CastMode) -> Self {
        match val {
            Unicast(addr) | Multicast(addr) => addr,
        }
    }
}

type AddrWithScope = (CastMode, ScopeId);

// scope_id will be overrided when converting from global addr
impl TryFrom<AddrWithScope> for Ipv6Scope {
    type Error = AnyError;
    fn try_from((addr, scope_id): AddrWithScope) -> Result<Self, Self::Error> {
        use std::net::Ipv6MulticastScope::*;
        match addr {
            u @ Unicast(addr) if addr.is_unicast_link_local() => Ok(Lan { addr: u, scope_id }),
            u @ Unicast(addr) if addr.is_unicast_global() => Ok(Wan(u)),
            m @ Multicast(addr)
                if matches!(
                    addr.multicast_scope(),
                    Some(Global | OrganizationLocal | SiteLocal | AdminLocal | RealmLocal)
                ) =>
            {
                Ok(Wan(m))
            }
            m @ Multicast(addr) if matches!(addr.multicast_scope(), Some(LinkLocal)) => {
                Ok(Lan { addr: m, scope_id })
            }
            _ => Err(AnyError::msg(
                "no convertion from linklocaltuple to ipv6scope for this value",
            )),
        }
    }
}

impl TryFrom<&SocketAddrV6> for Ipv6Scope {
    type Error = AnyError;

    fn try_from(val: &SocketAddrV6) -> Result<Self, Self::Error> {
        let addr: CastMode = (*val.ip()).try_into()?;
        let scope_id: ScopeId = val.scope_id().into();
        (addr, scope_id).try_into()
    }
}

/// always override the scope_id
impl From<Ipv6Scope> for Ipv6Addr {
    fn from(val: Ipv6Scope) -> Self {
        match val {
            Lan {
                addr: Unicast(addr),
                ..
            } => addr,
            Lan {
                addr: Multicast(addr),
                ..
            } => addr,
            Wan(Unicast(addr)) => addr,
            Wan(Multicast(addr)) => addr,
        }
    }
}

/// This is a newtype for convert (ipv6Scope,port) into SocketAddrV6
pub struct ScopeWithPort {
    pub addr: Ipv6Scope,
    pub port: u16,
}

impl ScopeWithPort {
    pub fn new(addr: Ipv6Scope, port: u16) -> Self {
        Self { addr, port }
    }
    pub async fn new_outbound(addr: Ipv6Scope, port: u16) -> Self {
        use crate::utils::NetworkInterfaceView as iface;
        let Ipv6Scope::Lan { addr, .. } = addr else {
            return Self::new(addr, port);
        };
        let ifaces = iface::instance().get().await;
        let linklocal_iface = ifaces.0.as_ref().unwrap();
        let lan = Ipv6Scope::Lan {
            addr,
            scope_id: linklocal_iface.scope_id().unwrap().into(),
        };
        Self::new(lan, port)
    }
}

impl From<ScopeWithPort> for SocketAddrV6 {
    fn from(val: ScopeWithPort) -> Self {
        let ScopeWithPort { addr, port } = val;
        match addr {
            Lan { addr, scope_id } => SocketAddrV6::new(addr.into(), port, 0, scope_id.into()),
            Wan(addr) => SocketAddrV6::new(addr.into(), port, 0, 0),
        }
    }
}

impl TryFrom<Interface> for Ipv6Scope {
    type Error = AnyError;
    fn try_from(val: Interface) -> Result<Self, Self::Error> {
        let IpAddr::V6(addr) = val.address() else {
            return Err(AnyError::msg("this is not a ipv6 addr"));
        };
        let scope_id = val
            .scope_id()
            .ok_or(AnyError::msg("scope does not exist"))?;
        ((*addr).try_into()?, scope_id.into()).try_into()
    }
}
