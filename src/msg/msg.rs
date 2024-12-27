use super::opt::OptMsg;
use crate::msg::hello::HelloMsg;
use std::str::FromStr;

#[derive(Debug)]
pub enum Message {
    Hello(HelloMsg),
    Opt(OptMsg),
}

impl FromStr for Message {
    type Err = ron::de::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.get(1..5) {
            Some("halo") => HelloMsg::from_str(s).map(Message::Hello),
            Some(hdr) if hdr.starts_with("opt") => OptMsg::from_str(s).map(Message::Opt),
            Some(s) => Err(ron::de::Error::InvalidIdentifier(s.to_string())),
            None => Err(ron::de::Error::Eof),
        }
    }
}
