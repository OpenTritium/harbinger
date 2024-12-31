use bitflags::bitflags;

bitflags! {
pub struct PeerStateCode: u8 {
        const CONNECTING = 0b00000001;
        const ESTABLISHED = 0b00000010;
    }
}