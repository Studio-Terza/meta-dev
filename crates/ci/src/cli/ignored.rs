invariants::ignored! {
    pub(crate) enum Ignored {
        Unparsed => "a line, name or page that does not parse as what the step looks for is not one of them",
        OtherShape => "a listing of another length is not that shape",
        BeyondRange => "a number past the target integer's range saturates, or names no exit code",
        NotUtf8 => "output that is not UTF-8 reads as empty in the message that reports it",
        NotUnicode => "an argument that is not Unicode names no command",
        Unmarked => "a directory with no readable tested marker has none",
        Outside => "a directory outside the workspace root has no place in it",
        Ungrepped => "git grep exits non-zero when nothing matches, which lists no files",
        Unfetched => "a tree that cannot be fetched leaves the plan whole, as unfetched",
    }
}
