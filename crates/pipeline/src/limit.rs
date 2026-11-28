#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Within,
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limit<Measured: PartialOrd>(Measured);

impl<Measured: PartialOrd> Limit<Measured> {
    pub const fn at(most: Measured) -> Limit<Measured> {
        Limit(most)
    }

    pub fn judged(&self, measured: &Measured) -> Verdict {
        if *measured > self.0 { Verdict::Over } else { Verdict::Within }
    }
}
