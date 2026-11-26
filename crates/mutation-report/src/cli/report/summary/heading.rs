use strum::Display;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Heading {
    #[strum(serialize = "Surviving mutants")]
    Surviving,
    #[strum(serialize = "Timed out mutants")]
    TimedOut,
    #[strum(serialize = "Tests that kill no mutant")]
    KillingNothing,
    #[strum(serialize = "Mutants killed while no golden test fails")]
    KilledWithoutGolden,
    #[strum(serialize = "Unviable mutants by crate")]
    Unviable,
}
