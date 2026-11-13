use lint_file::Swept;
use lint_long_exact_strings::LongExactStrings;

use lint_fixtures::{Case, Flagged, Lines, Located, Source, Sources, Text, Title};

const ESCAPED: &str = "fn t() {\n    assert_eq!(f(), \"a\\nb\");\n}\n";
const SPANNING: &str = "fn t() {\n    assert_eq!(\n        f(),\n        \"a\nb\"\n    );\n}\n";
const NOT_EQUAL: &str = "fn t() {\n    assert_ne!(\"a\\nb\".to_owned(), f());\n}\n";
const COMPARED: &str = "fn t() {\n    assert!(f() == \"a\\nb\");\n}\n";
const DIFFERENT: &str = "fn t() {\n    assert!(String::from(\"a\\nb\") != f());\n}\n";
const DEBUG: &str = "fn t() {\n    assert_eq!(f(), format!(\"{:?}\", g()));\n}\n";
const PRETTY: &str = "fn t() {\n    assert_eq!(f(), format!(\"{x:#?}\"));\n}\n";
const WRAPPED: &str = "fn t() {\n    assert_eq!(f(), Err(Some(&[\"a\", \"b\\n\"])));\n}\n";
const BOUND: &str = "fn t() {\n    let wanted = format!(\"{}\\n\", 1);\n    assert_eq!(f(), wanted);\n}\n";
const CONSTANT: &str = "const WANTED: &str = concat!(\"a\", \"\\n\");\nfn t() {\n    assert_eq!(f(), WANTED);\n}\n";
const TABLE: &str = "laws::table!(\n    t,\n    f,\n    [\n        1 => \"a\",\n        2 => \"a\\nb\",\n    ]\n);\n";
const IMPORTED_TABLE: &str = "use laws::table;\ntable!(t, f, [1 => format!(\"{:?}\", 1)]);\n";
const BYTES: &str = "fn t() {\n    assert_eq!(f(), b\"a\\n\");\n}\n";
const NESTED: &str = "fn t() {\n    proptest! {\n        fn u(a in 0..2) {\n            assert_eq!(f(a), \"a\\nb\");\n        }\n    }\n}\n";

const FIELD: &str = "fn t() {\n    assert_eq!(f(), Answer { out: \"a\\n\".to_owned() });\n}\n";
const GROUPED: &str = "fn t() {\n    assert_eq!(f(), (\"a\\nb\"));\n    assert_eq!(f(), (1, \"a\\nb\"));\n}\n";
const TABLE_EQUALS: &str = "laws::table!(t, f, [a == b => \"x\\ny\"]);\n";
const TABLE_SHIFT: &str = "laws::table!(t, f, [1 >> 2 => \"x\\ny\"]);\n";
const METHOD: &str = "impl T {\n    fn t() {\n        let w = \"a\\nb\";\n        assert_eq!(f(), w);\n    }\n}\n";
const ONE_LINE_BYTES: &str = "fn t() {\n    assert_eq!(f(), b\"ab\");\n}\n";
const DERIVED: &str =
    "fn t() {\n    assert_eq!(f(), \"a\\nb\".len());\n    assert_eq!(f(), line_count!(\"a\\nb\"));\n}\n";
const TABLE_THEN_GROUP: &str = "laws::table!(t, f, [1 => \"a\\nb\"], {});\n";
const ONE_LINE: &str = "fn t() {\n    assert_eq!(f(), \"a b\");\n    assert_eq!(f(), format!(\"{} b\", 1));\n}\n";
const INPUT: &str = "fn t() {\n    assert_eq!(f(\"a\\nb\"), 2);\n    assert!(g(\"a\\nb\"));\n}\n";
const DISPLAY: &str = "fn t() {\n    assert_eq!(f(), format!(\"{}\", g()));\n    assert_eq!(f(), \"{:?}\");\n}\n";
const ESCAPED_BRACE: &str = "fn t() {\n    assert_eq!(f(), format!(\"{{:?}}\"));\n}\n";
const MESSAGE: &str =
    "fn t() {\n    assert_eq!(f(), 1, \"{:?}\\n\", g());\n    assert!(f(), \"a\\nb {:?}\", g());\n}\n";
const ONE_LINE_TABLE: &str = "laws::table!(t, f, [\"a\\nb\" => 1]);\n";
const OTHER_MACRO: &str = "fn t() {\n    println!(\"a\\nb {:?}\", 1);\n    let a = format!(\"{:?}\", 1);\n}\n";
const ANOTHER_BODY: &str = "fn a() {\n    let w = \"a\\nb\";\n}\nfn t() {\n    assert_eq!(f(), w);\n}\n";
const TRAILING: &str = "fn t() {\n    assert_eq!(f(), format!(\"{}a\", 1));\n}\n";
const LESS: &str = "fn t() {\n    assert!(f() < \"a\\nb\");\n}\n";
const LOOPED: &str = "const A: &str = B;\nconst B: &str = A;\nfn t() {\n    let c = c;\n    assert_eq!(f(), A);\n    assert_eq!(f(), c);\n}\n";

const CASES: [Case; 33] = [
    Case::new(
        Title::new("escaped line break"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(ESCAPED))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("literal spanning lines"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(SPANNING))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("assert_ne and to_owned"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(NOT_EQUAL))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("assert with =="),
        &[Source::new(Located::new("tests/a.rs"), Text::new(COMPARED))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("assert with != and String::from"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(DIFFERENT))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("Debug formatting"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(DEBUG))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("pretty Debug of a named argument"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(PRETTY))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("inside wrappers"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(WRAPPED))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("bound to a local"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(BOUND))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("a constant"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(CONSTANT))],
        Flagged::new(&["tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("table expectation"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(TABLE))],
        Flagged::new(&["tests/a.rs:1"]),
    ),
    Case::new(
        Title::new("imported table"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(IMPORTED_TABLE))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("byte string"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(BYTES))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("inside another macro"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(NESTED))],
        Flagged::new(&["tests/a.rs:4"]),
    ),
    Case::new(
        Title::new("a field of a struct"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(FIELD))],
        Flagged::new(&["tests/a.rs:2"]),
    ),
    Case::new(
        Title::new("parenthesised and in a tuple"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(GROUPED))],
        Flagged::new(&["tests/a.rs:2", "tests/a.rs:3"]),
    ),
    Case::new(
        Title::new("a table row whose input holds =="),
        &[Source::new(Located::new("tests/a.rs"), Text::new(TABLE_EQUALS))],
        Flagged::new(&["tests/a.rs:1"]),
    ),
    Case::new(
        Title::new("a table row whose input holds >>"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(TABLE_SHIFT))],
        Flagged::new(&["tests/a.rs:1"]),
    ),
    Case::new(
        Title::new("inside a method"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(METHOD))],
        Flagged::new(&["tests/a.rs:4"]),
    ),
    Case::new(
        Title::new("bytes on one line"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(ONE_LINE_BYTES))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a table followed by another group"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(TABLE_THEN_GROUP))],
        Flagged::new(&["tests/a.rs:1"]),
    ),
    Case::new(
        Title::new("values derived from text by other methods and macros"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(DERIVED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a local of another body"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(ANOTHER_BODY))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("one letter after a placeholder"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(TRAILING))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("one line"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(ONE_LINE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("inputs are not compared"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(INPUT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("Display formatting and plain braces"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(DISPLAY))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("escaped braces"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(ESCAPED_BRACE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("the failure message"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(MESSAGE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("table input"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(ONE_LINE_TABLE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("not an assertion"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(OTHER_MACRO))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an ordering is not an equality"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(LESS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("cycles end"),
        &[Source::new(Located::new("tests/a.rs"), Text::new(LOOPED))],
        Flagged::new(&[]),
    ),
];

#[test]
fn assertions_that_pin_long_text_are_flagged() {
    assert_eq!(Sources::mismatches::<Swept<LongExactStrings>>(&CASES), Lines::default());
}
