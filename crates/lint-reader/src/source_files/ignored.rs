invariants::ignored! {
    pub(crate) enum Ignored {
        Unread => "a file that failed to read or parse declares no modules here; its failure is reported with the file",
        Unlisted => "a tree that failed to list has no paths; its failure stays in the listing",
        Outside => "a file outside its tree is shown as found",
        Unresolved => "a skipped declaration whose file cannot be resolved adds no test modules",
        NotAManifest => "a manifest that fails to read encloses no workspace",
        NotAPath => "an attribute that is not name = value names no path",
    }
}
