invariants::ignored! {
    pub enum Unlexed {
        NotThisGrammar => "text that does not lex or parse as this grammar is not this grammar; the caller reads it as absent or unspelled",
        OtherShape => "a lexing of another length or shape is not this form",
        OutOfRange => "digits past the target integer's range do not read as this number",
    }
}
