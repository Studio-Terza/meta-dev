invariants::ignored! {
    pub(crate) enum Ignored {
        NotAKind => "text whose kind is not an artifact kind is not an artifact",
        NotEntered => "text that names no way of entering may still name an origin; failing both, it is Unprovenanced",
    }
}
