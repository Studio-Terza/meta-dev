use crate::Measure;

pub trait Pipeline {
    type Unit;
    type Parsed: Measure;
    type Measured;
    type Act;

    fn parsed(&self, unit: Self::Unit) -> Self::Parsed;

    fn measured(&self, whole: Self::Parsed) -> Self::Measured;

    fn act(&self, measured: Self::Measured) -> Self::Act;

    fn run(&self, units: impl IntoIterator<Item = Self::Unit>) -> Self::Act {
        let whole = Self::Parsed::total(units.into_iter().map(|unit| self.parsed(unit)));
        self.act(self.measured(whole))
    }
}

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
