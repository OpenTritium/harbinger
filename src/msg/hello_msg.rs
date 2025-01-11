use crate::addr_v6::scope::Ipv6Scope;
use crate::env::env::get_env;
use crate::env::uid::Uid;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use crate::msg::msg::Msg;

// todo! 协议版本控制
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HelloMsg {
    halo: (),
    pub version: u8,
    pub host_id: Uid,
    pub addr: Ipv6Scope,
}

impl HelloMsg {
    pub fn new() -> Self {
        HelloMsg {
            halo: (),
            version: 0,
            host_id: get_env().host_id.clone(),
            addr: get_env().best_local_link().unwrap().clone(),
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

impl Into<Msg> for HelloMsg {
    fn into(self) -> Msg {
        Msg::Hello(self)
    }
}
