invariants::ignored! {
    pub(crate) enum Ignored {
        Untasked => "a task folder, task file or task time that cannot be read holds no running task",
        Unpicked => "a selection reply that does not parse as the asked JSON quotes nothing, so the stop proceeds",
        Unqueued => "a missing or unreadable queue file adds no queue section to a stalled stop",
        Unanswered => "a failed transcript lookup or model call never blocks a stop; the stop proceeds as before",
    }
}
