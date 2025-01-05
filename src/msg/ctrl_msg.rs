use crate::addr::ipv6_scope::Ipv6Scope;
use crate::env::uid::Uid;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CtrlMsg {
   ctrl: (),
    pub version: u8,
    pub ctrl_code: u8,
    pub host_id: Uid,
    pub addr: Ipv6Scope,
}

impl CtrlMsg {
    pub fn new(ctrl_code: u8, host_id: Uid, addr: Ipv6Scope) -> CtrlMsg {
        CtrlMsg {
            ctrl: (),
            version: 0,
            ctrl_code: ctrl_code,
            host_id,
            addr,
        }
    }
}


impl FromStr for CtrlMsg {
    type Err = ron::de::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ron::from_str::<CtrlMsg>(s)?)
    }
}

impl Display for CtrlMsg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ser = ron::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{}", ser)
    }
}
