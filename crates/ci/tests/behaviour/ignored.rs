invariants::ignored! {
    pub(crate) enum Ignored {
        Unplanned => "a plan or look that fails gives nothing, which the row reports",
        Unseeded => "a scratch file that fails to write leaves the step to fail, which the case reports",
        Unread => "a scratch file that cannot be read reads as empty, which the assertion on it reports",
    }
}

invariants::invariants! {
    pub(crate) enum Invariant {
        MemoryTakesADirectory => "an empty in-memory tree takes a scratch directory",
    }
}
