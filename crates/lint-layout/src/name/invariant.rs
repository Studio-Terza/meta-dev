invariants::invariants! {
    pub(crate) enum Invariant {
        WritingToAStringSucceeds => "formatting an identifier into a String cannot fail",
    }
}
