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
