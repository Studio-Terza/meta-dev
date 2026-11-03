use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Heading {
    Surviving,
    TimedOut,
    KillingNothing,
    KilledWithoutGolden,
}

impl fmt::Display for Heading {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Heading::Surviving => "Surviving mutants",
            Heading::TimedOut => "Timed out mutants",
            Heading::KillingNothing => "Tests that kill no mutant",
            Heading::KilledWithoutGolden => "Mutants killed while no golden test fails",
        })
    }
}
