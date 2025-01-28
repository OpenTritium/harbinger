use crate::addr_v6::Ipv6Scope;
use crate::utils::Uid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum Msg {
    Hello { host_id: Uid, addr: Ipv6Scope },
    Connect { host_id: Uid },
    Conflict,
}
