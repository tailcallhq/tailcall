mod can_track;
mod collect;
mod error;
mod event;
mod tracker;
use error::Result;
pub use event::{Event, EventKind};
pub use tracker::Tracker;
