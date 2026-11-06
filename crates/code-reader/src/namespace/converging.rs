use std::cmp::Ordering;

use super::boundary::Measure;

pub(super) trait Converging: Sized {
    fn measure(&self) -> Measure;

    fn advanced(&self) -> Self;

    fn converged(self) -> Self {
        let next = self.advanced();
        self.kept_or(next)
    }

    fn kept_or(self, next: Self) -> Self {
        match next.measure().cmp(&self.measure()) {
            Ordering::Equal => self,
            Ordering::Less | Ordering::Greater => next.converged(),
        }
    }
}
