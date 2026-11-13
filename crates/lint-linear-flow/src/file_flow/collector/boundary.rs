#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Order(usize);

impl Order {
    pub fn next(self) -> Order {
        Order(self.0 + 1)
    }
}
