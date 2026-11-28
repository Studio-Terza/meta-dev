invariants::ignored! {
    pub(crate) enum Ignored {
        Unnamed => "seconds that are negative, not a number or past chrono's range name no wall",
        Reversed => "a later wall measured from an earlier one spans nothing",
        Unbounded => "a span past chrono's range lands at the end of time",
    }
}
