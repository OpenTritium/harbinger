use Ipv6Scope::{Global, LinkLocal};
use Ipv6ScopeError::{MissingScope, RedundantScope, UnknownAddress};
use serde::{Deserialize, Serialize};
use std::net::{Ipv6Addr, SocketAddrV6};
use thiserror::Error;

#[derive(Debug, Eq, PartialEq, Hash, Serialize, Deserialize, Clone)]
pub struct ScopeId(u32);

impl From<u32> for ScopeId {
    fn from(val: u32) -> Self {
        Self(val)
    }
}

impl From<ScopeId> for u32 {
    fn from(val: ScopeId) -> Self {
        val.0
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, Eq, Hash, PartialEq)]
pub enum Ipv6Scope {
    LinkLocal(Ipv6Addr, ScopeId),
    Global(Ipv6Addr),
}

impl From<Ipv6Scope> for Ipv6Addr {
    fn from(val: Ipv6Scope) -> Self {
        match val { 
            LinkLocal(addr, _) => addr,
            Global(addr) => addr,
        }
    }
}

#[derive(Error, Debug)]
pub enum Ipv6ScopeError {
    #[error("Link-local address {0} is missing a matching scope_id")]
    MissingScope(Ipv6Addr),
    #[error("Redundant scope_id: {0:?}")]
    RedundantScope(ScopeId),
    #[error("Unknown address: {0}")]
    UnknownAddress(Ipv6Addr),
}
impl Ipv6Scope {
    pub fn try_from_ipv6addr(
        addr: Ipv6Addr,
        scope_id: Option<ScopeId>,
    ) -> Result<Ipv6Scope, Ipv6ScopeError> {
        match (addr, scope_id) {
            (addr, None) if addr.is_global() => Ok(Global(addr)),
            (addr, Some(s)) if addr.is_global() => Err(RedundantScope(s)),
            (addr, Some(scope_id)) if addr.is_unicast_link_local() => Ok(LinkLocal(addr, scope_id)),
            (addr, None) if addr.is_unicast_link_local() => Err(MissingScope(addr)),
            (addr, _) => Err(UnknownAddress(addr)),
        }
    }
    pub fn try_from_sockaddr_v6(sockaddr: SocketAddrV6) -> Result<Ipv6Scope, Ipv6ScopeError> {
        Self::try_from_ipv6addr(*sockaddr.ip(), Some(sockaddr.scope_id().into()))
    }
    pub fn into_sockaddr_v6(self, p: u16) -> SocketAddrV6 {
        match self {
            LinkLocal(addr, sid) => SocketAddrV6::new(addr, p, 0, sid.into()),
            Global(addr) => SocketAddrV6::new(addr, p, 0, 0),
        }
    }
}
