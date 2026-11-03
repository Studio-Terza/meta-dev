use std::ffi::OsString;

use graph_query::{Query, QueryFailure, Role, Term};
use lint_core::SourceFile;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_layout::Name;
use text_lines::Lines;
use transition_facts::Transitions;
use type_facts::{Facts, MethodShape};

const SEARCH: &str = "pub trait Index {\n    fn insert(&mut self, item: Item);\n    fn search(&self, query: &Query) -> Vec<Hit>;\n}\npub struct TextIndex;\nimpl Index for TextIndex {\n    fn insert(&mut self, item: Item) {}\n    fn search(&self, query: &Query) -> Vec<Hit> {}\n}\npub struct Plain;\nimpl Plain {\n    pub fn search_text(&self) {}\n}\nimpl<T: Index> Lookup for T {\n    fn find(&self, query: &Query) -> Vec<Hit> {}\n}\nmod store {\n    pub struct Records;\n    impl Records {\n        pub fn insert(&mut self, record: Record) {}\n        pub fn all(&self) -> Vec<Record> {}\n    }\n}\nimpl<T> Holder<T> {\n    pub fn get(&self) -> Option<T> {}\n}\n";
const FILES: &str = "pub trait Files {\n    fn read(&self, path: &Path) -> Result<Contents, Failure>;\n}\npub struct Disk;\nimpl Files for Disk {\n    fn read(&self, path: &Path) -> Result<Contents, Failure> {}\n}\n";
const LOOKUP_ROLE: &str = "fn lookup(&self, wanted: &Wanted) -> Vec<Found>";

struct Asked;

impl Asked {
    fn tree() -> Vec<SourceFile> {
        Sources::parsed(&[
            Source::new(Located::new("crates/search/src/lib.rs"), Text::new(SEARCH)),
            Source::new(Located::new("crates/disk/src/ports.rs"), Text::new(FILES)),
        ])
    }

    fn query(words: &[&str]) -> Result<Query, QueryFailure> {
        Query::try_from(words.iter().map(OsString::from).collect::<Vec<_>>().as_slice())
    }

    fn lines(words: &&[&str]) -> Vec<String> {
        let files = Self::tree();
        let facts = Facts::of_files(&files);
        let transitions = Transitions::of_files(&files);
        let query = Self::query(words).unwrap_or_else(|failure| unreachable!("the query reads: {failure}"));
        Lines::from(query.matches(&facts, &transitions).to_string().as_str()).map(ToOwned::to_owned).collect()
    }
}

laws::table!(terms_rank_names_above_methods_and_implementers_and_label_ports_and_stores, Asked::lines, [
    &["index"][..] => vec![
        "score 3: trait Index in search at crates/search/src/lib.rs:1; implemented by TextIndex".to_owned(),
    ],
    &["insert", "records"][..] => vec![
        "score 4: store Records in search at crates/search/src/lib.rs:18".to_owned(),
        "score 1: trait Index in search at crates/search/src/lib.rs:1; implemented by TextIndex".to_owned(),
    ],
    &["read", "files"][..] => vec![
        "score 4: port trait Files in disk at crates/disk/src/ports.rs:1; implemented by Disk".to_owned(),
        "score 2: port Disk in disk at crates/disk/src/ports.rs:4; implements Files".to_owned(),
    ],
    &["insert", "records", "--top", "1"][..] => vec!["score 4: store Records in search at crates/search/src/lib.rs:18".to_owned()],
    &["holder"][..] => vec!["score 3: generic impl Holder in search at crates/search/src/lib.rs:24".to_owned()],
    &["unrelated"][..] => vec!["no existing traits, generic impls, ports or stores match".to_owned()],
]);

#[test]
fn a_role_signature_finds_methods_of_the_same_shape_under_any_name() {
    assert_eq!(
        Asked::lines(&&["--role", LOOKUP_ROLE][..]),
        [
            "score 2: trait Index in search at crates/search/src/lib.rs:1; implemented by TextIndex",
            "    search fn(&self, _) -> many fits fn(&self, _) -> many",
            "score 2: generic impl Lookup for T in search at crates/search/src/lib.rs:14",
            "    find fn(&self, _) -> many fits fn(&self, _) -> many",
            "score 1: port trait Files in disk at crates/disk/src/ports.rs:1; implemented by Disk",
            "    read fn(&self, _) -> Result fits fn(&self, _) -> many",
            "score 1: port Disk in disk at crates/disk/src/ports.rs:4; implements Files",
            "    read fn(&self, _) -> Result fits fn(&self, _) -> many",
        ]
    );
}

#[test]
fn roots_are_read_from_the_in_flag() {
    let query = Asked::query(&["index", "--in", "crates/a", "--in", "crates/b"]);
    let roots: Vec<String> = query.iter().flat_map(Query::roots).map(|root| root.display().to_string()).collect();
    assert_eq!(roots, ["crates/a", "crates/b"]);
}

#[test]
fn malformed_arguments_are_refused_with_the_reason() {
    assert_eq!(Asked::query(&[]), Err(QueryFailure::Empty));
    assert_eq!(Asked::query(&["index", "--top"]), Err(QueryFailure::Unfinished));
    assert_eq!(Asked::query(&["index", "--top", "many"]), Err(QueryFailure::Top("many".to_owned())));
    let refused = Asked::query(&["--role", "not a signature"]).map_err(|failure| failure.to_string());
    assert!(refused.is_err_and(|reason| reason.contains("is not a Rust signature")));
}

#[test]
fn terms_match_camel_and_snake_words_and_plurals() {
    let Ok(term) = "Records".parse::<Term>();
    assert!(term.found_in(&Name::from("AllRecords")).is_some());
    assert!(term.found_in(&Name::from("read_record")).is_some());
    assert!(term.found_in(&Name::from("Recorder")).is_none());
    let Ok(short) = "bus".parse::<Term>();
    assert!(short.found_in(&Name::from("BuReader")).is_none());
}

#[test]
fn a_role_agrees_on_receiver_arity_self_arguments_and_return_shape() {
    let role: Result<Role, QueryFailure> = "fn f(&self, other: &Self) -> Option<Self>".parse();
    assert_eq!(
        role.map(|role| AsRef::<MethodShape>::as_ref(&role).to_string()),
        Ok("fn(&self, Self) -> Option".to_owned())
    );
}
