invariants::ignored! {
    pub(crate) enum Ignored {
        NotAManifest => "a manifest that fails to read places nothing and encloses no workspace",
        Uncanonical => "a named path with no canonical form is not found",
        Outside => "a file outside the package, or past its first part, has no place in it",
        Unnamed => "a directory that is neither src nor build.rs may still be a target directory, else it is other",
    }
}
