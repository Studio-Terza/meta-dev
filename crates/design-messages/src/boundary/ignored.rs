invariants::ignored! {
    pub(crate) enum Ignored {
        Unsliced => "a segment whose text is not a valid body is not stored as a row",
        Oversized => "a count past i64::MAX is stored as i64::MAX",
    }
}
