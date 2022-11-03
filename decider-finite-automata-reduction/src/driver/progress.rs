//! Track progress of the current prover and overall machine index.

use indicatif::{
    MultiProgress, ProgressBar, ProgressBarIter, ProgressFinish, ProgressIterator, ProgressStyle,
};
use std::borrow::Cow;

pub struct DeciderProgress {
    multi: MultiProgress,
    for_index: ProgressBar,
    prover_style: ProgressStyle,
}

pub trait DeciderProgressIterator
where
    Self: Sized + ExactSizeIterator,
{
    /// Wrap an iterator with a custom progress bar.
    fn decider_progress_with(
        self,
        progress: &DeciderProgress,
        name: impl Into<Cow<'static, str>>,
    ) -> ProgressBarIter<Self> {
        let prover_progress = progress
            .multi
            .add(ProgressBar::new(self.len() as u64))
            .with_style(progress.prover_style.clone())
            .with_message(name)
            .with_finish(ProgressFinish::AndLeave);
        self.progress_with(prover_progress)
    }
}

impl<S, T: ExactSizeIterator<Item = S>> DeciderProgressIterator for T {}

impl DeciderProgress {
    pub fn new(len: usize) -> DeciderProgress {
        let multi = MultiProgress::new();
        multi.set_move_cursor(true);
        let for_index = multi.add(ProgressBar::new(len as u64));
        let prover_style =
            ProgressStyle::with_template("{wide_bar} {pos:>7}/{len:7} ~{eta_precise:8} {msg} ")
                .unwrap();
        DeciderProgress {
            multi,
            for_index,
            prover_style,
        }
    }

    /// Print a log line above all progress bars.
    pub fn println<I: AsRef<str>>(&self, msg: I) -> std::io::Result<()> {
        self.multi.println(msg)
    }

    /// Update the overall (index) progress with a count of solved machines.
    pub fn set_solved(&self, solved: usize) {
        self.for_index.set_position(solved as u64);
    }

    /// Update the overall (index) progress after we find `n` solutions.
    pub fn solve(&self, n: usize) {
        self.for_index.inc(n as u64);
    }
}