invariants::ignored! {
    pub(crate) enum Ignored {
        BeyondRange => "a length or position past the target integer's range saturates, or advances nothing",
        Unformattable => "a Display that fails to write yields empty bytes",
        Unreadable => "metadata that cannot be read reads as a missing path",
        BeforeEpoch => "a file time before 1970 reads as the epoch itself",
    }
}
