use super::ScopeId;
use CastMode::*;
use Ipv6Scope::*;
use anyhow::Error as AnyError;
use anyhow::Ok;
use netif::Interface;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv6Addr, SocketAddrV6};

/// When the address is in unicast mode, `lan` represents link-local (fe80 addresses), and `wan` represents addresses belonging to a domain larger than link-local.
/// When the address is in multicast mode, `lan` exclusively represents multicast addresses within the link-local domain (excluding loopback addresses), whereas `wan` represents multicast addresses within the Global, Organization-Local, Site-Local, Admin-Local, or Realm-Local domains.
/// No other address modes should exist.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Ipv6Scope {
    Lan { addr: CastMode, scope_id: ScopeId }, // LinkLocal
    Wan(CastMode),                             // Wider scope than LinkLocal
}

impl TryFrom<Ipv6Addr> for CastMode {
    type Error = AnyError;
    fn try_from(val: Ipv6Addr) -> Result<Self, Self::Error> {
        match val {
            val if val.is_unicast() => Ok(CastMode::Unicast(val)),
            val if val.is_multicast() => Ok(CastMode::Multicast(val)),
            _ => Err(AnyError::msg(
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

impl From<CastMode> for Ipv6Addr {
    fn from(val: CastMode) -> Self {
        match val {
            Unicast(addr) | Multicast(addr) => addr,
        }
    }
}

type AddrWithScope = (CastMode, ScopeId);

// The scope_id will be overridden when converting from a global address.
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
                "The address is not a unicast address, and the multicast address does not belong to any multicast domain.",
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
    pub async fn new_outbound(addr: Ipv6Scope, port: u16) -> Self {
        use crate::utils::NetworkInterfaceView as iface;
        let Ipv6Scope::Lan { addr, .. } = addr else {
            return Self::new(addr, port);
        };
        let ifaces = iface::instance().get().await;
        let linklocal_iface = ifaces.0.as_ref().expect(
            "Failed to retrieve the link-local address interface in the application environment.",
        );
        let lan = Ipv6Scope::Lan {
            addr,
            scope_id: linklocal_iface
                .scope_id()
                .expect("The network interface has no associated scope ID.")
                .into(),
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
            return Err(AnyError::msg(
                "The interface is not an IPv6-enabled interface.",
            ));
        };
        let scope_id = val
            .scope_id()
            .ok_or(AnyError::msg("The interface has no associated scope ID."))?;
        ((*addr).try_into()?, scope_id.into()).try_into()
    }
}
