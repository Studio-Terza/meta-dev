use graph_reader::Reader;
use lint_file::ModelLint;
use lint_fixtures::{Lines, Located, Queried, Source, Sources, Text};
use lint_named_invariants::{NamedInvariants, UnnamedPanics};

const BARE_PANICS: &str = "fn f(x: u8) {\n    panic!(\"x\");\n    unreachable!();\n    todo!();\n    unimplemented!();\n    assert!(x > 1);\n    assert_eq!(x, 1);\n    debug_assert_ne!(x, 2);\n}\n";
const QUALIFIED: &str = "fn f() {\n    std::unreachable!();\n    core::panic!(\"x\");\n}\n";
const NAMED: &str = "use invariants::invariant;\nfn f(x: u8) {\n    invariant!(x > 1, Invariant::Positive);\n    invariant!(broken Invariant::Positive);\n}\n";
const IN_TESTS: &str = "#[cfg(test)]\nmod tests {\n    fn f() {\n        assert!(true);\n    }\n}\n";
const IN_MACRO_RULES: &str = "macro_rules! crash {\n    () => {\n        panic!(\"x\")\n    };\n}\n";
const OTHER_MACROS: &str = "fn f() {\n    println!(\"x\");\n    vec![1];\n}\n";

laws::table!(unnamed_panics_in_shipped_code_are_refused, Sources::flagged::<Queried<UnnamedPanics>>, [
    [Source::new(Located::new("a.rs"), Text::new(BARE_PANICS))]
        => Lines::from(["a.rs:2", "a.rs:3", "a.rs:4", "a.rs:5", "a.rs:6", "a.rs:7", "a.rs:8"]),
    [Source::new(Located::new("a.rs"), Text::new(QUALIFIED))] => Lines::from(["a.rs:2", "a.rs:3"]),
    [Source::new(Located::new("a.rs"), Text::new(NAMED))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(IN_TESTS))] => Lines::default(),
    [Source::new(Located::new("x/tests/run.rs"), Text::new(BARE_PANICS))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(IN_MACRO_RULES))] => Lines::default(),
    [Source::new(Located::new("a.rs"), Text::new(OTHER_MACROS))] => Lines::default(),
]);

laws::table!(named_invariants_are_counted_once_per_crate, Sources::flagged::<Queried<NamedInvariants>>, [
    [Source::new(Located::new("a.rs"), Text::new(NAMED))] => Lines::from(["a.rs:3"]),
    [Source::new(Located::new("a.rs"), Text::new(BARE_PANICS))] => Lines::default(),
]);

#[test]
fn a_crate_reports_its_named_invariant_count_with_the_report_fix() {
    let sources = [
        Source::new(Located::new("crates/x/src/lib.rs"), Text::new(NAMED)),
        Source::new(Located::new("crates/x/src/b.rs"), Text::new(NAMED)),
    ];
    let text = NamedInvariants::report(&Reader::read_files(&Sources::parsed(&sources))).to_string();
    assert!(
        text.contains(
            "crates/x/src/b.rs:3: crate x: 4 named invariants; report only: each named invariant marks a requirement a refinement type could carry instead"
        ),
        "{text}"
    );
}
