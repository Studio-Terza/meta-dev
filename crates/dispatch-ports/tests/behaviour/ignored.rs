invariants::ignored! {
    pub(crate) enum Ignored {
        Unparsed => "text that does not parse as a start time is not a case, and fails the roundtrip",
        Unstalled => "a dispatch, watch, lookup or settle that fails leaves no stall, which the test then reports",
    }
}
