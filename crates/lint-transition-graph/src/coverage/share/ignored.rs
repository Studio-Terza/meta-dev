invariants::ignored! {
    pub(crate) enum Ignored {
        BeyondRange => "a count past u32 saturates at its maximum in a share",
    }
}
