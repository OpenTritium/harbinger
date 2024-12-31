use crate::env::env::get_env;
use crate::env::uid::Uid;
use crate::peer::peer_state::PeerState;
use crate::peer::peer_state_code::PeerStateCode;
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
    pub fn new(opt_code: u8, host_id: Uid) -> OptMsg {
        OptMsg {
            opt: (),
            version: 0,
            opt_code,
            host_id,
        }
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
        let ser = ron::to_string(self).map_err(|_| fmt::Error)?;
        write!(f, "{}", ser)
    }
}

impl OptMsg {
    pub fn gen_msg_by_state(e: &PeerState) -> Self {
        let uid = get_env().host_id.clone();
        match e {
            PeerState::Connect(_) => OptMsg::new(PeerStateCode::CONNECTING.bits(), uid),
            PeerState::Establish(_) => OptMsg::new(PeerStateCode::ESTABLISHED.bits(), uid),
        }
    }
}
