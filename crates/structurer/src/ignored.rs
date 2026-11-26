invariants::ignored! {
    pub(crate) enum Ignored {
        Unparsed => "a word that does not parse as the name wanted is not that name, and the reply is Misread",
        OtherShape => "a call with another number of arguments is not this call",
        SelfEdge => "a sample whose node would refine itself proposes nothing",
        Unspelled => "text that is neither a node id nor a spelled label is Unlabelled",
    }
}
