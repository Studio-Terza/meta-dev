invariants::ignored! {
    pub(crate) enum Ignored {
        Panicked => "a panicking resolver thread stands for a failed resolve",
        Negative => "a negative offset is not a case",
        Failing => "disk and memory agree when both fail, whatever their failure text",
    }
}
