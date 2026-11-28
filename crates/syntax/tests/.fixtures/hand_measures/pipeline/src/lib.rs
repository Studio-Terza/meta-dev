pub trait Measure: Default {
    fn joined(self, other: Self) -> Self;
}

impl Measure for usize {
    fn joined(self, other: usize) -> usize {
        self.saturating_add(other)
    }
}
