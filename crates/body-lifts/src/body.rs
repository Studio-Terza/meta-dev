#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Body {
    Written,
    Mapped,
    Focused,
    Routed,
}
