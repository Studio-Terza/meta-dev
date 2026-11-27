invariants::ignored! {
    pub(crate) enum Ignored {
        Oversized => "a count past i64::MAX is stored as i64::MAX",
        Unasked => "a lead sentence that is not a valid body leaves a short owner reply quoted alone",
    }
}
