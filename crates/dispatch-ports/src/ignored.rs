invariants::ignored! {
    pub(crate) enum Ignored {
        BeforeEpoch => "a clock set before the Unix epoch reads as the epoch",
        BeyondRange => "seconds past the i64 range saturate at its maximum",
        Unfed => "a program that exits before reading all its input is judged by its exit and output alone",
    }
}
