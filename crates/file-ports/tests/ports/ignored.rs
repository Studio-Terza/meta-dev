invariants::ignored! {
    pub(crate) enum Ignored {
        Panicked => "a panicking resolver thread stands for a failed resolve",
        Failing => "disk and memory agree when both fail, whatever their failure text",
    }
}
