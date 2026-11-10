use lint_core::SourceFile;
use lint_fixtures::{Sources, Text};
use lint_layout::{PathTail, Word, words};
use lint_paths::{Cover, Ground, Paths};

const TARGET: [Word<'static>; 2] = words!["fs", "remove_dir_all"];

struct Reads;

impl Reads {
    fn spelled(files: &[SourceFile]) -> Vec<String> {
        Paths::read(files, &words!["remove_dir_all"])
            .iter()
            .filter(|used| used.spelling(PathTail::new(&TARGET)).is_some())
            .map(|used| used.place().to_string())
            .collect()
    }

    fn of_one(text: &&str) -> Vec<String> {
        Self::spelled(&[Sources::file("a.rs", Text::new(text))])
    }

    fn of_two(pair: &[[&str; 2]; 2]) -> Vec<String> {
        Self::spelled(&pair.map(|[path, text]| Sources::file(path, Text::new(text))))
    }
}

laws::table!(
    forms_that_reach_the_target_are_found,
    Reads::of_one,
    [
        "fn f() {\n    std::fs::remove_dir_all(\"x\");\n}\n" => vec!["a.rs:2".to_owned()],
        "use std::fs;\nfn f() {\n    fs::remove_dir_all(\"x\");\n}\n" => vec!["a.rs:3".to_owned()],
        "fn f() {\n    fs::remove_dir_all(\"x\");\n}\nuse std::fs;\n" => vec!["a.rs:2".to_owned()],
        "use std::fs::*;\nfn f() {\n    remove_dir_all(\"x\");\n}\n" => vec!["a.rs:3".to_owned()],
        "use std::fs::{self as disk};\nfn f() {\n    disk::remove_dir_all(\"x\");\n}\n" => vec!["a.rs:3".to_owned()],
        "type Disk = std::fs::File;\nuse std::fs as disk;\nuse self::disk as again;\nfn f() {\n    again::remove_dir_all(\"x\");\n}\n"
            => vec!["a.rs:5".to_owned()],
        "mod inner {\n    pub use std::fs::remove_dir_all;\n}\nfn f() {\n    inner::remove_dir_all(\"x\");\n}\n"
            => vec!["a.rs:5".to_owned()],
        "mod inner {\n    pub fn f() {\n        super::wipe(\"x\");\n    }\n}\nuse std::fs::remove_dir_all as wipe;\n"
            => vec!["a.rs:3".to_owned()],
        "fn f() {\n    vec![std::fs::remove_dir_all(\"x\"); 2];\n}\n" => vec!["a.rs:2".to_owned()],
        "fn f() {\n    cache.remove_dir_all();\n}\n" => Vec::<String>::new(),
        "fn f() {\n    <std::fs as Tr>::remove_dir_all(\"x\");\n}\n" => Vec::<String>::new(),
        "fn f() {\n    <std as fs>::remove_dir_all(\"x\");\n}\n" => Vec::<String>::new(),
        "use foo::foo;\nfn f() {\n    foo::remove_dir_all(\"x\");\n}\n" => Vec::<String>::new(),
        "use std::fs::remove_dir_all;\nfn f(cache: u8) {\n    vec![cache.remove_dir_all()];\n}\n"
            => Vec::<String>::new(),
        "use std::fs::remove_dir_all;\nfn f() {\n    vec![1, remove_dir_all(\"x\")];\n}\n"
            => vec!["a.rs:3".to_owned()],
        "mod a {\n    pub use std::fs::remove_dir_all;\n}\nmod b {\n    fn remove_dir_all(_x: u8) {}\n    fn f() {\n        remove_dir_all(1);\n    }\n}\n"
            => Vec::<String>::new(),
    ]
);

#[test]
fn import_cycles_end() {
    let text = "mod a {\n    pub use super::b::x;\n}\nmod b {\n    pub use super::a::x;\n}\nfn f() {\n    a::x();\n}\n";
    assert_eq!(Reads::of_one(&text), Vec::<String>::new());
}

laws::table!(
    a_re_export_reaches_other_files_of_its_crate_and_modules_without_a_file_are_entered,
    Reads::of_two,
    [
        [
            ["crates/x/src/lib.rs", "pub use std::fs::remove_dir_all;\nmod user;\n"],
            ["crates/x/src/user.rs", "fn f() {\n    crate::remove_dir_all(\"x\");\n}\n"],
        ] => vec!["crates/x/src/user.rs:2".to_owned()],
        [
            ["crates/one/src/lib.rs", "pub use std::fs::remove_dir_all;\n"],
            ["crates/two/src/lib.rs", "fn f() {\n    crate::remove_dir_all(\"x\");\n}\n"],
        ] => Vec::<String>::new(),
        [
            ["crates/x/src/lib.rs", "mod a;\nfn f() {\n    a::b::remove_dir_all(\"x\");\n}\n"],
            ["crates/x/src/a/b.rs", "pub use std::fs::remove_dir_all;\n"],
        ] => vec!["crates/x/src/lib.rs:3".to_owned()],
    ]
);

#[test]
fn a_use_remembers_where_it_stands_and_whether_a_guard_holds_it() {
    let text = "mod ports {\n    pub struct G;\n    impl Drop for G {\n        fn drop(&mut self) {\n            std::fs::remove_dir_all(\"x\");\n        }\n    }\n    pub fn bare() {\n        std::fs::remove_dir_all(\"x\");\n    }\n}\nfn open() {\n    std::fs::remove_dir_all(\"x\");\n}\n";
    let file = Sources::file("a.rs", Text::new(text));
    let seen: Vec<(Ground, Cover)> = Paths::read(&[file], &words!["remove_dir_all"])
        .iter()
        .filter(|used| used.spelling(PathTail::new(&TARGET)).is_some())
        .map(|used| (used.ground(), used.cover()))
        .collect();
    assert_eq!(seen, vec![(Ground::Ports, Cover::Guarded), (Ground::Ports, Cover::Bare), (Ground::Open, Cover::Bare)]);
}

laws::table!(
    only_paths_that_can_reach_the_wanted_names_are_kept,
    |text: &&str| Paths::read(&[Sources::file("a.rs", Text::new(text))], &words!["remove_dir_all"]).len(),
    [
        "fn f(x: u8) {\n    Vec::new();\n    x;\n    std::fs::remove_dir_all(\"x\");\n}\n" => 1,
        "use std::fs::remove_dir_all as wipe;\nfn f() {\n    wipe(\"x\");\n    other(1);\n}\n" => 1,
        "use std::fs::*;\nfn f() {\n    other(1);\n}\n" => 1,
        "use std::fs::remove_dir_all as wipe;\nmod m {\n    pub fn f() {\n        super::wipe(\"x\");\n        other(1);\n    }\n}\n" => 1,
        "fn f() {\n    other(1);\n    Vec::new();\n}\n" => 0,
    ]
);
