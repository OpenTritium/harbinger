use crate::addr::ipv6_scope::Ipv6Scope;
use crate::env::env::get_env;
use crate::env::uid::Uid;
use crate::peer::peer_state::PeerState;
use crate::peer::peer_state::PeerState::Connect;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

// todo! 协议版本控制
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloMsg {
    halo: (),
    pub version: u8,
    pub host_id: Uid,
    pub addr: Ipv6Scope,
}

impl HelloMsg {
    pub fn new(host: &Ipv6Scope) -> Self {
        HelloMsg {
            halo: (),
            version: 0,
            host_id: get_env().host_id.clone(),
            addr: host.clone(),
        }
    }
}

impl FromStr for HelloMsg {
    type Err = ron::de::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ron::from_str::<Self>(s)?)
    }
}

impl Display for HelloMsg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ser = ron::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{}", ser)
    }
}

impl From<HelloMsg> for (Uid, PeerState) {
    fn from(msg: HelloMsg) -> Self {
        (msg.host_id, Connect(msg.addr))
    }
}
