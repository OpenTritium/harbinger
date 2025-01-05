use bitflags::bitflags;

// 报文只会用三个，全部都用来挂在服务
bitflags! {
pub struct PeerCtrlCode: u8 {
        const HELLO = 0b00000001;
        const CONNECT = 0b00000010;
        const ESTABLISH = 0b00000100;
        const TRANSFERRING = 0b00001000;
        const UNREACHABLE = 0b00010000;
    }
}
