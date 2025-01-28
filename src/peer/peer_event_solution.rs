use crate::msg::{EventSender,ParcelSender};
use crate::peer::peer_event::PeerEvent;
use crate::peer::peer_event_flags::PeerEventFlags;
use futures::future::BoxFuture;
use std::error::Error;

use super::peer_event_handler::PeerStateTable;


pub type FutureResult<T> = BoxFuture<'static, Result<T, anyhow::Error>>;
pub type SolutionClosure =
    dyn FnOnce(ParcelSender, PeerStateTable, EventSender) -> FutureResult<()>;
pub type DispatchResult = Result<Box<SolutionClosure>, anyhow::Error>;
pub trait PeerEventSolution: Send + Sync {
    fn dispatch_solution(&self, event: Box<PeerEvent>) -> DispatchResult;
    fn interest(&self) -> &'static PeerEventFlags;
}
