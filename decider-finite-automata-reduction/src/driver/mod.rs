//! Features that help humans operate the decider, like distributed computing and progress tracking.

mod node_crunch;
mod progress;

pub use self::node_crunch::{process_remote, run_node};
pub use progress::{DeciderProgress, DeciderProgressIterator};
