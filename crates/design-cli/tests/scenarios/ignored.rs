invariants::ignored! {
    pub(crate) enum Ignored {
        Unnamed => "an event that does not serialise names no hook, so the check finds it missing",
        Unrun => "a scenario that fails to start or run gives no answer, which the case reports",
        NotUtf8 => "output that is not UTF-8 gives no answer, which the case reports",
    }
}
