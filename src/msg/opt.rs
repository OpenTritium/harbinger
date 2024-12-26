use crate::env::get_env;
use crate::peer::PeerFutureState;
use crate::uid::Uid;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OptMsg {
    opt: (),
    pub version: u8,
    pub opt_code: u8,
    pub host_id: Uid,
}

impl OptMsg {
    pub fn new(version: u8, opt_code: u8, host_id: Uid) -> OptMsg {
        OptMsg {
            opt: (),
            version,
            opt_code,
            host_id,
        }
    }
}

bitflags! {
pub struct OptCode: u8 {
        const CONNECTING = 0b0001;
        const ESTABLISHING = 0b0010;
        const DISPOSING = 0b0100;
        const DISCONNECTING = 0b1000;
    }
}

impl FromStr for OptMsg {
    type Err = ron::de::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ron::from_str::<OptMsg>(s)?)
    }
}

impl Display for OptMsg {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let serialized = ron::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{}", serialized)
    }
}

impl OptMsg {
    pub fn gen_msg_by_enum(e: &PeerFutureState) -> Self {
        let uid = get_env().host_id.clone();
        match e {
            PeerFutureState::Connect(_) => OptMsg::new(0, OptCode::CONNECTING.bits(), uid),
            PeerFutureState::Establish(_) => OptMsg::new(0, OptCode::ESTABLISHING.bits(), uid),
            PeerFutureState::Dispose(_) => OptMsg::new(0, OptCode::DISPOSING.bits(), uid),
            PeerFutureState::Disconnect(_) => OptMsg::new(0, OptCode::DISCONNECTING.bits(), uid),
        }
    }
}
