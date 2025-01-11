use crate::msg::msg::Msg;
use bytes::BytesMut;
use std::str::{FromStr, from_utf8};
use tokio_util::codec::{Decoder, Encoder};


//todo 错误处理
pub struct MsgCodec {}

impl Decoder for MsgCodec {
    type Item = Msg;
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }
        let msg =
            Msg::from_str(from_utf8(src.split().freeze().to_vec().as_ref()).unwrap()).unwrap();
        Ok(Some(msg))
    }
}

impl Encoder<Msg> for MsgCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: Msg, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(item.to_string().as_bytes());
        Ok(())
    }
}

impl MsgCodec {
    pub fn new() -> Self {
        Self {}
    }
}
