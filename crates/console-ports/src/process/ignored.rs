invariants::ignored! {
    pub(crate) enum Ignored {
        Unwritable => "an answer that cannot be written to standard output or error exits with failure",
        Unlocatable => "an executable that cannot locate itself reads as the empty path, which no file is",
    }
}
