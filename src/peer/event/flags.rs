use bitflags::bitflags;

// 用于状态表中标识会话属性，以及挂载服务处理器
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PeerEventFlags: u8 {
        const HELLO = 0b00000001;
        const CONNECT = 0b00000010;
        const ESTABLISHED = 0b00000100;
        const CONFLICT = 0b00000011;// 很显然你不可能同时hello和connect
    }
}
