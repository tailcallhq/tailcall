mod can_track;
mod collect;
mod dispatch;
mod error;
mod event;
pub use dispatch::Tracker;
use error::Result;
pub use event::{Event, EventKind};
