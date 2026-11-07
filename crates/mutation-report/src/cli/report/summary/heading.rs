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
        match self {
            Self::Surviving => formatter.write_str("Surviving mutants"),
            Self::TimedOut => formatter.write_str("Timed out mutants"),
            Self::KillingNothing => formatter.write_str("Tests that kill no mutant"),
            Self::KilledWithoutGolden => formatter.write_str("Mutants killed while no golden test fails"),
        }
    }
}
