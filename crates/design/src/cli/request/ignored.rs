invariants::ignored! {
    pub(crate) enum Ignored {
        Unparsed => "a word that does not parse as the lens or hook event asked for is a usage failure",
        OtherShape => "arguments of another count or shape are a usage failure",
        NotAUuid => "a prefix that is not a whole uuid is looked up as a prefix",
        Unclosed => "an unclosed quote names no segment here; it is reported as unclosed when sought",
        Missed => "a citation or quote that is not found contributes nothing here; it is reported as missed",
        Unlooked => "a start-time lookup that failed records no start; the watch reports the failure",
    }
}
