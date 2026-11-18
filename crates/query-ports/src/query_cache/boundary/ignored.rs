invariants::ignored! {
    pub(crate) enum Ignored {
        Complained => "a preparation that failed shows nothing; Answer::reported complains with its failure",
        Absent => "a cache file that cannot be read is treated as absent and written afresh",
        Outside => "a path outside the root is shown as given",
    }
}
