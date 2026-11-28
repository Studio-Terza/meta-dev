invariants::ignored! {
    pub(crate) enum Ignored {
        Unparsed => "tokens that do not parse as the syntax asked for are not that syntax, so they name nothing here",
        BeyondRange => "a token longer than usize ends the scan of comment lines",
        Outside => "a path outside the tree is shown as it is",
        Missing => "a candidate module file that cannot be resolved is not that module's file",
    }
}
