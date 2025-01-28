use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PeerEventFlags: u8 {
        const HELLO = 0b00000001;
        const CONNECTED = 0b00000010;
    }
}