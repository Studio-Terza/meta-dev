invariants::ignored! {
    pub(crate) enum Ignored {
        Unspawned => "a helper thread that cannot start leaves its work to the threads that did",
        Unwritable => "a refusal that cannot be written to standard error has nowhere left to go; the exit code still says Refused",
        Unknown => "an unknown parallelism reads as one worker",
        Unclaimed => "a walked file whose head was not gathered has no claim",
    }
}
