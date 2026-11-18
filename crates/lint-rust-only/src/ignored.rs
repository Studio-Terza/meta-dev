invariants::ignored! {
    pub(crate) enum Ignored {
        Unread => "a file that cannot be read has no language to judge",
        Undecoded => "bytes that are not UTF-8 have no text to read a language or shebang from",
        Unparsed => "text that does not parse as a shebang, fence or path part names no language",
        BeyondRange => "a scan limit past usize scans the whole file",
    }
}
