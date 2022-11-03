//! Features that help humans operate the decider, like distributed computing and progress tracking.

mod progress;

pub use progress::{DeciderProgress, DeciderProgressIterator};
