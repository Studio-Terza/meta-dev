invariants::ignored! {
    pub(crate) enum Ignored {
        Unseeded => "a fixture file or directory that fails to write leaves the step to fail, which the case reports",
        Unrun => "a fixture step that fails gives nothing, which the case reports",
        Refused => "a text the check or parse refuses is filtered out, or is the None the case expects",
        Unparsed => "output that does not parse as a design line or briefing is left out, which the case reports",
        Unread => "a node that cannot be read is not placed, which the case reports",
        Unsettled => "a run whose release failed counts nothing, which the case reports",
    }
}
