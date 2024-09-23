mod can_track;
mod collect;
mod error;
mod event;
mod dispatch;
use error::Result;
pub use event::{Event, EventKind};
pub use dispatch::Tracker;
