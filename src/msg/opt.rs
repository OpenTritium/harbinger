use crate::env::get_env;
use crate::peer::future_state::PeerFutureState;
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
    pub fn new(opt_code: u8, host_id: Uid) -> OptMsg {
        OptMsg {
            opt: (),
            version:0,
            opt_code,
            host_id,
        }
    }
}

bitflags! {
pub struct OptCode: u8 {
        const CONNECTING = 0b00000001;
        const ESTABLISHING = 0b00000010;
        const DISPOSING = 0b00000100;
        const DISCONNECTING = 0b00001000;
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
    pub fn gen_msg_by_state(e: &PeerFutureState) -> Self {
        let uid = get_env().host_id.clone();
        match e {
            PeerFutureState::Connect(_) => OptMsg::new(OptCode::CONNECTING.bits(), uid),
            PeerFutureState::Establish(_) => OptMsg::new( OptCode::ESTABLISHING.bits(), uid),
            PeerFutureState::Dispose(_) => OptMsg::new( OptCode::DISPOSING.bits(), uid),
            PeerFutureState::Disconnect(_) => OptMsg::new( OptCode::DISCONNECTING.bits(), uid),
        }
    }
}
