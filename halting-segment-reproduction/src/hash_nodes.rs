use crate::*;

impl PartialEq for SegmentCells {
    fn eq(&self, other: &SegmentCells) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        for i in 0..self.0.len() {
            if self.0[i] != other.0[i] {
                return false;
            }
        }
        return true;
    }
}
impl Eq for SegmentCells {}

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
