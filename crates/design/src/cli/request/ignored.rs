invariants::ignored! {
    pub(crate) enum Ignored {
        Unparsed => "a word that does not parse as the lens or hook event asked for is a usage failure",
        OtherShape => "arguments of another count or shape are a usage failure",
        NotAUuid => "a prefix that is not a whole uuid is looked up as a prefix",
        Unclosed => "an unclosed quote names no segment here; it is reported as unclosed when sought",
        Missed => "a citation or quote that is not found contributes nothing here; it is reported as missed",
        Unlooked => "a start-time lookup that failed records no start; the watch reports the failure",
        Untasked => "a task folder, task file or task time that cannot be read holds no running task",
        Unpicked => "a selection reply that does not parse as the asked JSON quotes nothing, so the stop proceeds",
        Unanswered => "a failed transcript lookup or model call never blocks a stop; the stop proceeds as before",
        Unrecalled => "a failed node lookup or model call injects no design nodes into the prompt",
    }
}
