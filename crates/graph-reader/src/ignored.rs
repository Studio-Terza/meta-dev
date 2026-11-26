invariants::ignored! {
    pub(crate) enum Ignored {
        Unparsed => "tokens or source that do not parse as the syntax asked for are not that syntax, so they declare nothing",
        OtherShape => "a path of another length is not a canonical std path",
        Unknown => "an unknown parallelism reads as one share",
    }
}
