use super::ctrl_msg::CtrlMsg;
use crate::msg::hello_msg::HelloMsg;
use std::str::FromStr;

#[derive(Debug)]
pub enum Message {
    Hello(HelloMsg),
    Ctrl(CtrlMsg),
}

impl FromStr for Message {
    type Err = ron::de::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.get(1..5) {
            Some("halo") => HelloMsg::from_str(s).map(Message::Hello),
            Some("ctrl") => CtrlMsg::from_str(s).map(Message::Ctrl),
            Some(s) => Err(ron::de::Error::InvalidIdentifier(s.to_string())),
            None => Err(ron::de::Error::Eof),
        }
    }
}
