use lint_arbitrary_newtypes::ArbitraryNewtypes;
use lint_fixtures::{Located, Source, Sources, Text};

const NONE: [&str; 0] = [];
const COUNT: &str = "#[derive(Newtype, Clone, Copy)]\npub struct Count(usize);\n";
const NAMED: &str = "#[derive(Clone, Newtype)]\nstruct Label {\n    text: String,\n}\n";
const QUALIFIED: &str = "#[derive(newtype::Newtype)]\npub struct Paths(Vec<PathBuf>);\n";
const NESTED: &str = "mod inner {\n    #[derive(Newtype)]\n    pub(crate) struct Seconds(f64);\n}\n";
const GENERATED: &str = "#[derive(Newtype, Arbitrary, Clone, Copy)]\npub struct Count(usize);\n";
const GENERATED_ELSEWHERE: &str = "#[derive(Newtype)]\n#[derive(newtype::Arbitrary)]\npub struct Count(usize);\n";
const BORROWED: &str = "#[derive(Newtype)]\npub struct Title(&'static str);\n";
const BORROWING: &str = "#[derive(Newtype)]\npub struct Text<'text>(Cow<'text, str>);\n";
const PARSER: &str = "#[derive(Newtype)]\npub struct Parser(fn(&str) -> usize);\n";
const PLAIN: &str = "#[derive(Clone, Copy)]\npub struct Count(usize);\n";

laws::table!(a_newtype_that_owns_its_value_needs_a_generator, Sources::flagged::<ArbitraryNewtypes>, [
    [Source::new(Located::new("a.rs"), Text::new(COUNT))] => ["a.rs:2"],
    [Source::new(Located::new("a.rs"), Text::new(NAMED))] => ["a.rs:2"],
    [Source::new(Located::new("a.rs"), Text::new(QUALIFIED))] => ["a.rs:2"],
    [Source::new(Located::new("a.rs"), Text::new(NESTED))] => ["a.rs:3"],
]);

laws::table!(generated_borrowing_and_plain_types_pass, Sources::flagged::<ArbitraryNewtypes>, [
    [Source::new(Located::new("a.rs"), Text::new(GENERATED))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(GENERATED_ELSEWHERE))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(BORROWED))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(BORROWING))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(PARSER))] => NONE,
    [Source::new(Located::new("a.rs"), Text::new(PLAIN))] => NONE,
    [Source::new(Located::new("x/tests/a.rs"), Text::new(COUNT))] => NONE,
]);

#[test]
fn the_message_names_the_type_and_the_derive_to_add() {
    let report =
        Sources::report::<ArbitraryNewtypes>(&[Source::new(Located::new("a.rs"), Text::new(COUNT))]).to_string();
    assert!(report.contains("Count has no generator; add `Arbitrary` to its derives"));
}
