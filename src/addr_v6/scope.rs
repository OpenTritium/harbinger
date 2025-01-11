use crate::addr_v6::scope_id::ScopeId;
use crate::env::env::get_env;
use Ipv6Scope::{Global, LinkLocal};
use serde::{Deserialize, Serialize};
use std::net::{Ipv6Addr, SocketAddrV6};
use thiserror::Error;
//todo 转换函数不能依赖环境，不过可以包装
#[derive(Serialize, Deserialize, Debug, Clone, Eq, Hash, PartialEq)]
pub enum Ipv6Scope {
    LinkLocal { addr: Ipv6Addr, scope_id: ScopeId },
    Global(Ipv6Addr),
}

pub type LinkLocalTuple = (Ipv6Addr, ScopeId);
impl TryFrom<LinkLocalTuple> for Ipv6Scope {
    type Error = Ipv6ScopeError;

    fn try_from(val: LinkLocalTuple) -> Result<Self, Self::Error> {
        match val {
            (addr, sid) if addr.is_global() => Err(Ipv6ScopeError::RedundantScope(sid)),
            (addr, scope_id) if addr.is_unicast_link_local() => Ok(LinkLocal { addr, scope_id }),
            (addr, _) => Err(Ipv6ScopeError::UnknownAddress(addr)),
        }
    }
}

impl TryFrom<Ipv6Addr> for Ipv6Scope {
    type Error = Ipv6ScopeError;

    fn try_from(val: Ipv6Addr) -> Result<Self, Self::Error> {
        if val.is_global() {
            Ok(Global(val))
        } else {
            Err(Ipv6ScopeError::InvalidAddressType(val))
        }
    }
}

impl TryFrom<&SocketAddrV6> for Ipv6Scope {
    type Error = Ipv6ScopeError;

    fn try_from(val: &SocketAddrV6) -> Result<Self, Self::Error> {
        Self::try_from((*val.ip(), val.scope_id().into()))
    }
}
impl From<Ipv6Scope> for Ipv6Addr {
    fn from(val: Ipv6Scope) -> Self {
        match val {
            LinkLocal { addr, .. } => addr,
            Global(addr) => addr,
        }
    }
}

pub struct ScopeWithPort {
    pub scope: Ipv6Scope,
    pub port: u16,
}
impl From<ScopeWithPort> for SocketAddrV6 {
    fn from(val: ScopeWithPort) -> Self {
        match val {
            ScopeWithPort {
                scope: LinkLocal { addr, scope_id },
                port,
            } => SocketAddrV6::new(addr, port, 0, scope_id.into()),
            ScopeWithPort {
                scope: Global(addr),
                port,
            } => SocketAddrV6::new(addr, port, 0, 0),
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
    #[error("invalid address type: {0:?}")]
    InvalidAddressType(Ipv6Addr),
}
impl Ipv6Scope {
    pub fn replace_scope_id(self) -> Result<Ipv6Scope, Ipv6ScopeError> {
        if let LinkLocal { scope_id, .. } = get_env().best_local_link().unwrap() {
            if let LinkLocal { addr, .. } = self {
                return Ok(LinkLocal { addr, scope_id });
            }
        }
        Err(Ipv6ScopeError::InvalidAddressType(self.into()))
    }
}
