use crate::{msg::msg::Msg, utils::Env};
use bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Default)]
pub struct MsgCodec;

impl MsgCodec {
    const HEADER_LEN: usize = size_of::<u16>() + size_of::<u8>();
}

impl Decoder for MsgCodec {
    type Item = Msg;
    type Error = bincode::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < MsgCodec::HEADER_LEN {
            return Ok(None);
        }
        let msg_len = u16::from_be_bytes([src[0], src[1]]) as usize;
        let protocol_version = src[2];
        if src.len() < msg_len {
            src.reserve(msg_len - src.len());
            return Ok(None);
        }
        if protocol_version != Env::instance().protocol_version {
            src.advance(msg_len);
            return Ok(None);
        }
        let msg = bincode::deserialize(&src.split_to(msg_len)[MsgCodec::HEADER_LEN..])?;
        Ok(Some(msg))
    }
}

impl Encoder<Msg> for MsgCodec {
    type Error = bincode::Error;

    fn encode(&mut self, item: Msg, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let msg = bincode::serialize(&item)?;
        dst.extend(
            ((msg.len() + Self::HEADER_LEN) as u16)
                .to_be_bytes()
                .iter()
                .copied()
                .chain([Env::instance().protocol_version].iter().copied())
                .chain(msg.into_iter()),
        );
        Ok(())
    }
}
