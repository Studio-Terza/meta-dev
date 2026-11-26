#![deny(clippy::wildcard_enum_match_arm)]

use grammar_git::GitFile;

const OBJECT: &str = "0123456789abcdef0123456789abcdef01234567";

const PACKED: &str = "# pack-refs with: peeled fully-peeled sorted\n\
                      0123456789abcdef0123456789abcdef01234567 refs/heads/main\n\
                      ^89abcdef0123456789abcdef0123456789abcdef\n\
                      fedcba9876543210fedcba9876543210fedcba98 refs/heads/other\n";

laws::table!(
    a_detached_head_or_a_ref_file_is_one_object_name_on_its_first_line,
    |text: &&str| GitFile::read(text).object().map(|object| object.as_ref().to_owned()),
    [
        "0123456789abcdef0123456789abcdef01234567\n" => Some(OBJECT.to_owned()),
        "0123456789abcdef0123456789abcdef01234567" => Some(OBJECT.to_owned()),
        "0123456789abcdef0123456789abcdef0123456\n" => None,
        "0123456789abcdef0123456789abcdef012345678\n" => None,
        "ref: refs/heads/main\n" => None,
        "" => None,
    ]
);

laws::table!(
    a_symbolic_head_names_its_ref_after_ref_and_a_space,
    |text: &&str| GitFile::read(text).symbolic().map(|name| name.as_ref().to_owned()),
    [
        "ref: refs/heads/design/verifier\n" => Some("refs/heads/design/verifier".to_owned()),
        "ref:refs/heads/main\n" => None,
        "ref: refs/heads/main extra\n" => None,
        "0123456789abcdef0123456789abcdef01234567\n" => None,
    ]
);

laws::table!(
    a_worktree_git_file_names_its_git_directory_after_gitdir_and_a_space,
    |text: &&str| GitFile::read(text).linked().map(|path| path.as_ref().to_owned()),
    [
        "gitdir: /repo/.git/worktrees/wt\n" => Some("/repo/.git/worktrees/wt".to_owned()),
        "gitdir:/repo/.git\n" => None,
        "ref: refs/heads/main\n" => None,
    ]
);

laws::table!(
    a_common_directory_file_is_one_path,
    |text: &&str| GitFile::read(text).common().map(|path| path.as_ref().to_owned()),
    [
        "../..\n" => Some("../..".to_owned()),
        "../.. x\n" => None,
        "" => None,
    ]
);

laws::table!(
    packed_refs_name_the_object_of_the_ref_asked_for_and_skip_comments_and_peeled_lines,
    |head: &&str| {
        let named = GitFile::read(&format!("ref: {head}\n")).symbolic();
        named.and_then(|name| GitFile::read(PACKED).packed(&name)).map(|object| object.as_ref().to_owned())
    },
    [
        "refs/heads/main" => Some(OBJECT.to_owned()),
        "refs/heads/other" => Some("fedcba9876543210fedcba9876543210fedcba98".to_owned()),
        "refs/heads/absent" => None,
    ]
);
