mod codec;
mod event_adapter;
mod msg;
mod socket;
mod splitter;

pub use event_adapter::{AdapterIo, MsgEventAdapter};
pub use msg::Msg;
pub use socket::{Parcel, ParcelReceiver, ParcelSender};
pub use splitter::{EventReceiver, EventSender, MsgSplitter};
