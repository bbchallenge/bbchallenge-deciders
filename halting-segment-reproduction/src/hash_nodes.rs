use crate::*;

impl std::hash::Hash for SegmentCells {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        for cell in self.0.iter() {
            cell.hash(state);
        }
        state.finish();
    }
}
