invariants::ignored! {
    pub(crate) enum Ignored {
        Unresolved => "a skipped declaration whose file cannot be resolved adds no test modules",
        NotAPath => "an attribute that is not name = value names no path",
        Unsized => "a file whose size cannot be read weighs nothing when files are dealt to readers; its read failure is reported when it is parsed",
        HungUp => "a reader whose coordinator or helper has hung up has nothing more to send or receive; the side that hung up reports why",
    }
}
