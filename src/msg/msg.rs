use super::ctrl_msg::CtrlMsg;
use crate::msg::hello_msg::HelloMsg;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Msg {
    Hello(HelloMsg),
    Ctrl(CtrlMsg),
}

impl FromStr for Msg {
    type Err = ron::de::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.get(1..5) {
            Some("halo") => HelloMsg::from_str(s).map(Msg::Hello),
            Some("ctrl") => CtrlMsg::from_str(s).map(Msg::Ctrl),
            Some(s) => Err(ron::de::Error::InvalidIdentifier(s.to_string())),
            None => Err(ron::de::Error::Eof),
        }
    }
}

impl Display for Msg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Msg::Hello(h) => {
                write!(f, "{}", h)
            }
            Msg::Ctrl(c) => {
                write!(f, "{}", c)
            }
        }
    }
}
