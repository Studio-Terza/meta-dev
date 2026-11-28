use pipeline::Measure;

#[derive(Default, Measure)]
pub struct Derived(usize);

#[derive(Default)]
pub struct Hand(usize);

impl pipeline::Measure for Hand {
    fn joined(self, other: Hand) -> Hand {
        Hand(self.0.joined(other.0))
    }
}
