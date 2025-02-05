use CastMode::*;
use Ipv6Scope::*;
use anyhow::Error as AnyError;
use anyhow::Ok;
use anyhow::anyhow;
use netif::Interface;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::net::{IpAddr, Ipv6Addr, SocketAddrV6};

/// When the address is in unicast mode, `lan` represents link-local (fe80 addresses), and `wan` represents addresses belonging to a domain larger than link-local.
/// When the address is in multicast mode, `lan` exclusively represents multicast addresses within the link-local domain (excluding loopback addresses), whereas `wan` represents multicast addresses within the Global, Organization-Local, Site-Local, Admin-Local, or Realm-Local domains.
/// No other address modes should exist.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Ipv6Scope {
    Lan { addr: CastMode, scope_id: u32 }, // LinkLocal
    Wan(CastMode),                         // Wider scope than LinkLocal
}

impl Ipv6Scope {
    pub fn scope_id(&self) -> Option<u32> {
        let Lan { scope_id, .. } = self else {
            return None;
        };
        Some(*scope_id)
    }
    pub fn modify_scope_id(&mut self, scope_id: Option<u32>) {
        if let (Some(new_scope_id), Ipv6Scope::Lan { scope_id, .. }) = (scope_id, self) {
            *scope_id = new_scope_id;
        }
    }
}

impl std::fmt::Display for Ipv6Scope{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lan { addr, scope_id } => write!(f, "{addr}%{scope_id}"),
            Wan(addr) => write!(f, "{addr}"),
        }
    }
}

impl TryFrom<Ipv6Addr> for CastMode {
    type Error = AnyError;
    fn try_from(val: Ipv6Addr) -> Result<Self, Self::Error> {
        match val {
            val if val.is_unicast() => Ok(CastMode::Unicast(val)),
            val if val.is_multicast() => Ok(CastMode::Multicast(val)),
            _ => Err(anyhow!(
                "The IPv6 address is neither a unicast nor multicast address.",
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum CastMode {
    Unicast(Ipv6Addr),
    Multicast(Ipv6Addr),
}

impl Display for CastMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unicast(addr) | Multicast(addr) => write!(f, "{addr}"),
        }
    }
}

impl From<CastMode> for Ipv6Addr {
    fn from(val: CastMode) -> Self {
        match val {
            Unicast(addr) | Multicast(addr) => addr,
        }
    }
}

type AddrWithScope = (CastMode, Option<u32>);

// The scope_id will be overridden when converting from a global address.
impl TryFrom<AddrWithScope> for Ipv6Scope {
    type Error = AnyError;
    fn try_from((addr, scope_id): AddrWithScope) -> Result<Self, Self::Error> {
        use std::net::Ipv6MulticastScope::*;
        match addr {
            u @ Unicast(addr) if addr.is_unicast_link_local() => Ok(Lan {
                addr: u,
                scope_id: scope_id.ok_or(anyhow!("missing scope."))?,
            }),
            u @ Unicast(addr) if addr.is_unicast_global() => Ok(Wan(u)),
            m @ Multicast(addr)
                if matches!(
                    addr.multicast_scope(),
                    Some(Global | OrganizationLocal | SiteLocal | AdminLocal | RealmLocal)
                ) =>
            {
                Ok(Wan(m))
            }
            m @ Multicast(addr) if matches!(addr.multicast_scope(), Some(LinkLocal)) => Ok(Lan {
                addr: m,
                scope_id: scope_id.ok_or(anyhow!("missing scope."))?,
            }),
            Unicast(addr) => Err(anyhow!("Invalid unicast address scope: {addr}.")),
            Multicast(addr) => Err(anyhow!("Multicast address {addr} has invalid scope.")),
        }
    }
}

impl TryFrom<&SocketAddrV6> for Ipv6Scope {
    type Error = AnyError;

    fn try_from(val: &SocketAddrV6) -> Result<Self, Self::Error> {
        let addr: CastMode = (*val.ip()).try_into()?;
        (addr, Some(val.scope_id())).try_into() // 就算是全局地址，下面也会自动忽略scope
    }
}

// The scope identifier will be overridden during address conversion
impl From<Ipv6Scope> for Ipv6Addr {
    fn from(val: Ipv6Scope) -> Self {
        match val {
            Lan {
                addr: Unicast(addr) | Multicast(addr),
                ..
            }
            | Wan(Unicast(addr) | Multicast(addr)) => addr,
        }
    }
}

/// This newtype facilitates the conversion of an (Ipv6Scope, port) tuple into a SocketAddrV6.
pub struct ScopeWithPort {
    pub addr: Ipv6Scope,
    pub port: u16,
}

impl ScopeWithPort {
    pub fn new(addr: Ipv6Scope, port: u16) -> Self {
        Self { addr, port }
    }
}

impl From<ScopeWithPort> for SocketAddrV6 {
    fn from(val: ScopeWithPort) -> Self {
        let ScopeWithPort { addr, port } = val;
        match addr {
            Lan { addr, scope_id } => SocketAddrV6::new(addr.into(), port, 0, scope_id),
            Wan(addr) => SocketAddrV6::new(addr.into(), port, 0, 0),
        }
    }
}

impl TryFrom<&Interface> for Ipv6Scope {
    type Error = AnyError;
    fn try_from(val: &Interface) -> Result<Self, Self::Error> {
        let IpAddr::V6(addr) = val.address() else {
            return Err(anyhow!("The interface is not an IPv6-enabled interface.",));
        };
        ((*addr).try_into()?, val.scope_id()).try_into()
    }
}
