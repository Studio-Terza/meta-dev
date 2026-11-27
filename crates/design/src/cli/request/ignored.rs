invariants::ignored! {
    pub(crate) enum Ignored {
        Untasked => "a task folder, task file or task time that cannot be read holds no running task",
        Unpicked => "a selection reply that does not parse as the asked JSON quotes nothing, so the stop proceeds",
        Unanswered => "a failed transcript lookup or model call never blocks a stop; the stop proceeds as before",
        Unchosen => "a choice reply that does not parse descends every child, since a missed topic costs more than an extra one",
        Unlogged => "a session log line that does not parse as a known entry holds no stop",
    }
}
