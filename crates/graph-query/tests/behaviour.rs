use std::ffi::OsString;

use graph_query::{Query, QueryFailure, Role, Term};
use lint_core::SourceFile;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_layout::Name;
use lint_transition_graph::{Extraction, TypeSystem};
use text_lines::Lines;
use transition_facts::Transitions;
use type_facts::{Facts, MethodShape};

const SEARCH: &str = "pub trait Index {\n    fn insert(&mut self, item: Item);\n    fn search(&self, query: &Query) -> Vec<Hit>;\n}\npub struct TextIndex;\nimpl Index for TextIndex {\n    fn insert(&mut self, item: Item) {}\n    fn search(&self, query: &Query) -> Vec<Hit> {}\n}\npub struct Plain;\nimpl Plain {\n    pub fn search_text(&self) {}\n}\nimpl<T: Index> Lookup for T {\n    fn find(&self, query: &Query) -> Vec<Hit> {}\n}\nmod store {\n    pub struct Records;\n    impl Records {\n        pub fn insert(&mut self, record: Record) {}\n        pub fn all(&self) -> Vec<Record> {}\n    }\n}\nimpl<T> Holder<T> {\n    pub fn get(&self) -> Option<T> {}\n}\n";
const FILES: &str = "pub trait Files {\n    fn read(&self, path: &Path) -> Result<Contents, Failure>;\n}\npub struct Disk;\nimpl Files for Disk {\n    fn read(&self, path: &Path) -> Result<Contents, Failure> {}\n}\n";
const LOOKUP_ROLE: &str = "fn lookup(&self, wanted: &Wanted) -> Vec<Found>";

const ITEMS: [Source; 3] = [
    Source::new(
        Located::new("crates/shop/src/lib.rs"),
        Text::new(
            "mod till;\npub trait Priced {\n    fn price(&self) -> Cents;\n}\npub struct Cents;\npub struct Basket(till::Line);\nimpl<T: Clone> Priced for Vec<T> {\n    fn price(&self) -> Cents {}\n}\nimpl Priced for u32 {\n    fn price(&self) -> Cents {}\n}\n",
        ),
    ),
    Source::new(
        Located::new("crates/shop/src/till.rs"),
        Text::new(
            "use crate::{Cents, Priced};\npub struct Line(Cents);\nimpl Priced for Line {\n    fn price(&self) -> Cents {}\n}\npub struct Receipt;\n",
        ),
    ),
    Source::new(Located::new("crates/bank/src/lib.rs"), Text::new("pub struct Ledger(shop::Basket);\n")),
];

const CHAINS: [Source; 3] = [
    Source::new(Located::new("crates/shop/src/lib.rs"), Text::new("pub struct Cents;\n")),
    Source::new(
        Located::new("crates/alpha/src/lib.rs"),
        Text::new("pub struct Start(Mid);\npub struct Mid(Mid2);\npub struct Mid2(shop::Cents);\n"),
    ),
    Source::new(Located::new("crates/zeta/src/lib.rs"), Text::new("pub struct Start(shop::Cents);\n")),
];

const AUDIT: [Source; 3] = [
    Source::new(
        Located::new("crates/shop/src/lib.rs"),
        Text::new(
            "pub struct Cents;\npub trait Display {}\npub trait Priced {}\nimpl Priced for [u8] {}\nimpl Priced for (Cents, u8) {}\nimpl Priced for dyn Display {}\nimpl Priced for fn(u8) -> u8 {}\nimpl Priced for u32 {}\n",
        ),
    ),
    Source::new(
        Located::new("crates/bank/src/lib.rs"),
        Text::new(
            "pub trait Audited {}\npub struct Ledger;\nimpl Audited for shop::Cents {}\nimpl Audited for Ledger {}\nimpl std::fmt::Display for Box<Ledger> {}\n",
        ),
    ),
    Source::new(Located::new("crates/bank/src/other.rs"), Text::new("pub struct Other;\n")),
];

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

    fn answered(words: &&[&str]) -> Vec<String> {
        Self::answered_in(&ITEMS, words)
    }

    fn audited(words: &&[&str]) -> Vec<String> {
        Self::answered_in(&AUDIT, words)
    }

    fn chained(words: &&[&str]) -> Vec<String> {
        Self::answered_in(&CHAINS, words)
    }

    fn answered_in(sources: &[Source], words: &&[&str]) -> Vec<String> {
        let files = Sources::parsed(sources);
        let extraction = Extraction::from(files.as_slice());
        let system = TypeSystem::built_from(&files, &extraction);
        let query = Self::query(words).unwrap_or_else(|failure| unreachable!("the query reads: {failure}"));
        Lines::from(query.reply(&system).to_string().as_str()).map(ToOwned::to_owned).collect()
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
    assert_eq!(Asked::query(&["--users"]), Err(QueryFailure::Unfinished));
    assert_eq!(Asked::query(&["--path", "Ledger"]), Err(QueryFailure::Unfinished));
    assert_eq!(Asked::query(&["index", "--top", "many"]), Err(QueryFailure::Top("many".to_owned())));
    assert_eq!(Asked::query(&["--implementor", "Files"]), Err(QueryFailure::Flag("--implementor".to_owned())));
    assert_eq!(Asked::query(&["index", "-x"]), Err(QueryFailure::Flag("-x".to_owned())));
    assert_eq!(Asked::query(&["--defined", "--implementor"]), Err(QueryFailure::Flag("--implementor".to_owned())));
    assert_eq!(Asked::query(&["--uses", "--users", "Ledger"]), Err(QueryFailure::Unfinished));
    assert_eq!(Asked::query(&["--path", "Ledger", "--top", "3"]), Err(QueryFailure::Unfinished));
    let refused = Asked::query(&["--role", "not a signature"]).map_err(|failure| failure.to_string());
    assert!(refused.is_err_and(|reason| reason.contains("is not a Rust signature")));
}

#[test]
fn the_flag_failures_list_the_flags_of_the_table() {
    assert_eq!(
        QueryFailure::Flag("--x".to_owned()).to_string(),
        "`--x` is not a flag: the flags are --in, --role, --top, --defined, --uses, --users, --implementers, --cluster, --exports and --path"
    );
    assert_eq!(
        QueryFailure::Unfinished.to_string(),
        "--in, --role, --top, --defined, --uses, --users, --implementers, --cluster and --exports each take a value, and --path takes two"
    );
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

laws::table!(impls_in_another_crate_resolve_to_the_indexed_type_keep_the_trait_path_and_spell_every_self_type, Asked::audited, [
    &["--implementers", "Audited", "--uses", "Cents", "--users", "Audited"][..] => vec![
        "implementers of Audited:".to_owned(),
        "  bank::Audited at crates/bank/src/lib.rs:1".to_owned(),
        "    Cents at crates/shop/src/lib.rs:1".to_owned(),
        "    Ledger at crates/bank/src/lib.rs:2".to_owned(),
        "what Cents uses:".to_owned(),
        "  shop::Cents at crates/shop/src/lib.rs:1".to_owned(),
        "    Audited at crates/bank/src/lib.rs:1".to_owned(),
        "users of Audited:".to_owned(),
        "  bank::Audited at crates/bank/src/lib.rs:1".to_owned(),
        "    Cents at crates/shop/src/lib.rs:1".to_owned(),
        "    Ledger at crates/bank/src/lib.rs:2".to_owned(),
    ],
    &["--implementers", "Display"][..] => vec![
        "implementers of Display:".to_owned(),
        "  shop::Display at crates/shop/src/lib.rs:2".to_owned(),
        "    none".to_owned(),
    ],
    &["--implementers", "Priced"][..] => vec![
        "implementers of Priced:".to_owned(),
        "  shop::Priced at crates/shop/src/lib.rs:3".to_owned(),
        "    impl for (Cents, u8) at crates/shop/src/lib.rs:5".to_owned(),
        "    impl for [u8] at crates/shop/src/lib.rs:4".to_owned(),
        "    impl for dyn Display at crates/shop/src/lib.rs:6".to_owned(),
        "    impl for fn(u8) -> u8 at crates/shop/src/lib.rs:7".to_owned(),
        "    impl for u32 at crates/shop/src/lib.rs:8".to_owned(),
    ],
]);

laws::table!(item_questions_answer_from_the_containment_view, Asked::answered, [
    &["--defined", "Line"][..] => vec![
        "definitions of Line:".to_owned(),
        "  shop::till::Line at crates/shop/src/till.rs:2".to_owned(),
    ],
    &["--users", "Cents", "--implementers", "Priced"][..] => vec![
        "users of Cents:".to_owned(),
        "  shop::Cents at crates/shop/src/lib.rs:5".to_owned(),
        "    Line at crates/shop/src/till.rs:2".to_owned(),
        "    Priced at crates/shop/src/lib.rs:2".to_owned(),
        "implementers of Priced:".to_owned(),
        "  shop::Priced at crates/shop/src/lib.rs:2".to_owned(),
        "    Line at crates/shop/src/till.rs:2".to_owned(),
        "    impl for Vec at crates/shop/src/lib.rs:7".to_owned(),
        "    impl for u32 at crates/shop/src/lib.rs:10".to_owned(),
    ],
    &["--uses", "Line"][..] => vec![
        "what Line uses:".to_owned(),
        "  shop::till::Line at crates/shop/src/till.rs:2".to_owned(),
        "    Cents at crates/shop/src/lib.rs:5".to_owned(),
        "    Priced at crates/shop/src/lib.rs:2".to_owned(),
    ],
    &["--users", "Cents", "--top", "1"][..] => vec![
        "users of Cents:".to_owned(),
        "  shop::Cents at crates/shop/src/lib.rs:5".to_owned(),
        "    Line at crates/shop/src/till.rs:2".to_owned(),
        "    and 1 more".to_owned(),
    ],
    &["--cluster", "Line", "--defined", "Absent"][..] => vec![
        "cluster of Line:".to_owned(),
        "  shop::till::Line at crates/shop/src/till.rs:2".to_owned(),
        "    in shop::till; its users lie within crate shop; beside Receipt".to_owned(),
        "definitions of Absent:".to_owned(),
        "  no type or trait named Absent".to_owned(),
    ],
    &["--cluster", "Line", "--exports", "till"][..] => vec![
        "cluster of Line:".to_owned(),
        "  shop::till::Line at crates/shop/src/till.rs:2".to_owned(),
        "    in shop::till; its users lie within crate shop; beside Receipt".to_owned(),
        "exports of till:".to_owned(),
        "  shop::till".to_owned(),
        "    Line at crates/shop/src/till.rs:2; used by Basket".to_owned(),
    ],
    &["--exports", "bank"][..] => vec![
        "exports of bank:".to_owned(),
        "  crate bank".to_owned(),
        "    none".to_owned(),
    ],
    &["--exports", "shop", "--top", "0"][..] => vec![
        "exports of shop:".to_owned(),
        "  crate shop".to_owned(),
        "    and 1 more".to_owned(),
    ],
    &["--path", "Ledger", "Cents", "--top", "2"][..] => vec![
        "use path from Ledger to Cents:".to_owned(),
        "  bank::Ledger at crates/bank/src/lib.rs:1".to_owned(),
        "  shop::Basket at crates/shop/src/lib.rs:6".to_owned(),
        "  and 2 more".to_owned(),
    ],
    &["--path", "Ledger", "Cents", "--path", "Cents", "Ledger", "--defined", "Absent"][..] => vec![
        "use path from Ledger to Cents:".to_owned(),
        "  bank::Ledger at crates/bank/src/lib.rs:1".to_owned(),
        "  shop::Basket at crates/shop/src/lib.rs:6".to_owned(),
        "  shop::till::Line at crates/shop/src/till.rs:2".to_owned(),
        "  shop::Cents at crates/shop/src/lib.rs:5".to_owned(),
        "use path from Cents to Ledger:".to_owned(),
        "  none".to_owned(),
        "definitions of Absent:".to_owned(),
        "  no type or trait named Absent".to_owned(),
    ],
]);

laws::table!(a_path_is_the_shortest_from_any_type_of_the_starting_name, Asked::chained, [
    &["--path", "Start", "Cents"][..] => vec![
        "use path from Start to Cents:".to_owned(),
        "  zeta::Start at crates/zeta/src/lib.rs:1".to_owned(),
        "  shop::Cents at crates/shop/src/lib.rs:1".to_owned(),
    ],
]);
