invariants::ignored! {
    pub(crate) enum Ignored {
        Unparsed => "a site line whose number does not parse is not a violation",
        NotUtf8 => "output that is not UTF-8 reads as empty, which the assertion on it then reports",
        Unknown => "an unknown parallelism reads as one worker, as the runner reads it",
        Unreadable => "a file that cannot be read holds no occurrences",
        Unwritten => "a fixture that cannot be written judges nothing, which the table row then reports",
        Outside => "a path outside the scratch root is shown whole",
    }
}
