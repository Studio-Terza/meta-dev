invariants::ignored! {
    pub(crate) enum Ignored {
        Refused => "a query the arguments refuse is reported by the refusal, so the run has nothing to answer",
        Unlocated => "a workspace that cannot be located is reported once, as Unlocated; elsewhere it has no workspaces",
        Unspawned => "a helper thread that cannot start leaves its work to the threads that did",
        Unwritable => "a refusal that cannot be written to standard error has nowhere left to go; the exit code still says Refused",
        Unknown => "an unknown parallelism reads as one worker",
        Unclaimed => "a walked file whose head was not gathered has no claim",
        Unhashed => "a lint binary that cannot be read has no hash, so the run neither recalls nor remembers files",
        Unremembered => "a store that cannot be read or written leaves every file to be file-linted afresh",
        Unrecalled => "a store that no longer parses leaves every file to be file-linted afresh",
        Unviolated => "a file with no violations for a lint files no lines for it",
    }
}
