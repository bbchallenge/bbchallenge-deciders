//! Track progress of the current prover and overall machine index.

use indicatif::{
    MultiProgress, ProgressBar, ProgressBarIter, ProgressFinish, ProgressIterator, ProgressState,
    ProgressStyle,
};
use std::{borrow::Cow, fmt::Write, time::Duration};

const INDEX: &str =
    "{pos:>8}+{left:>8}               {wide_bar:.green/red} All Deciders: {elapsed_precise:12}";
const PROVER: &str =
    "{pos:>8}/{len:>8} ~{eta_precise:12} {wide_bar} {msg:12}: {elapsed_precise:12}";

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
        let len = self.len();
        self.progress_with(progress.prover_progress(len, name))
    }
}

impl<S, T: ExactSizeIterator<Item = S>> DeciderProgressIterator for T {}

impl DeciderProgress {
    pub fn new(len: usize) -> DeciderProgress {
        let multi = MultiProgress::new();
        multi.set_move_cursor(true);
        let index_style = ProgressStyle::with_template(INDEX).unwrap().with_key(
            "left",
            |state: &ProgressState, w: &mut dyn Write| {
                let pos = state.pos();
                write!(w, "{:>7}", state.len().unwrap_or(pos) - pos).unwrap()
            },
        );
        let prover_style = ProgressStyle::with_template(PROVER).unwrap();
        let pb = ProgressBar::new(len as u64)
            .with_style(index_style)
            .with_finish(ProgressFinish::Abandon);
        let for_index = multi.add(pb);
        for_index.enable_steady_tick(Duration::from_millis(500));
        DeciderProgress {
            multi,
            for_index,
            prover_style,
        }
    }

    pub fn prover_progress(&self, len: usize, name: impl Into<Cow<'static, str>>) -> ProgressBar {
        let bar = self
            .multi
            .add(ProgressBar::new(len as u64))
            .with_style(self.prover_style.clone())
            .with_message(name)
            .with_finish(ProgressFinish::AndLeave);
        bar.enable_steady_tick(Duration::from_millis(500));
        bar
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

    /// Finish everything.
    pub fn finish(&self) {
        self.for_index.abandon();
    }
}
