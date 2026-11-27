invariants::ignored! {
    pub(crate) enum Ignored {
        Untasked => "a task folder, task file or task time that cannot be read holds no running task",
        Unpicked => "a selection reply that does not parse as the asked JSON quotes nothing, so the stop proceeds",
        Unqueued => "a missing or unreadable queue file adds no queue section to a stalled stop",
        Untyped => "a last message that does not parse as DONE or BLOCKED is untyped and is sent back once",
        Unrun => "a gh or git call that cannot be started leaves its step unread, and an unread step is escalated",
        Unheaded => "a worktree head that does not read as a full commit sha counts as unpushed, so the worktree is kept",
        Unviewed => "a pull request view that does not parse as the asked fields leaves its step unread, and that is escalated",
        Unconfigured => "a missing or unreadable workspace local.toml key names nothing, so a step that needs it is escalated",
        Unspecified => "a first commit without a well-formed Spec trailer leaves the pull request unopened, and that is escalated",
        Unlisted => "a check list that cannot be read counts as no checks yet, so the hook polls again until its cap",
        Untitled => "a spec file that cannot be read gives no detail heading, so the pull request is unopened and that is escalated",
        Unremembered =>"a failed-check record that cannot be read or written counts no repeat failure",
        Unanswered =>"a failed transcript lookup or model call never blocks a stop; the stop proceeds as before",
    }
}
