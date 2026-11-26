invariants::ignored! {
    pub(crate) enum Ignored {
        Oversized => "a count past i64::MAX is stored as i64::MAX",
    }
}
