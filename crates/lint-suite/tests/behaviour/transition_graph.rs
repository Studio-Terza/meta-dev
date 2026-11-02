use lint_fixtures::{Located, Source, Sources, Text};
use lint_transition_graph::TransitionGraph;

struct Graphed;

impl Graphed {
    fn of(sources: &[Source]) -> String {
        Sources::report::<TransitionGraph>(sources).to_string()
    }

    fn at(path: &'static str, text: &str) -> Source {
        Source::new(Located::new(path), Text::new(Box::leak(text.to_owned().into_boxed_str())))
    }

    fn shows(pair: &[&str]) -> bool {
        match pair {
            [text, expected] => Self::of(&[Self::at("crates/x/src/lib.rs", text)]).contains(expected),
            _ => false,
        }
    }
}

const CONVERSIONS: &str = "struct Raw;\nstruct Policy;\nimpl From<Raw> for Policy {\n    fn from(raw: Raw) -> Self {}\n}\nimpl Policy {\n    fn parsed(raw: &Raw) -> Result<Policy, Error> {}\n}\n";
const PRIMITIVES: &str = "struct A;\nimpl A {\n    fn is_ready(&self) -> bool {}\n    fn label(&self) -> String {}\n    fn run(&mut self) {}\n    fn count(&self) -> Option<Vec<usize>> {}\n}\n";
const FOREIGN: &str = "struct A;\nimpl A {\n    fn path(&self) -> PathBuf {}\n    fn each<T>(&self, item: T) -> A {}\n    fn pair(&self) -> (A, A) {}\n}\n";
const WRAPPED: &str = "struct A;\nstruct B;\nimpl A {\n    fn all(&self) -> impl Iterator<Item = &B> {}\n    fn many(items: &[B]) -> Box<Self> {}\n    fn shared(&self) -> Rc<RefCell<B>> {}\n}\n";
const TESTED: &str = "struct A;\n#[cfg(test)]\nmod tests {\n    impl A {\n        fn f(&self) -> bool {}\n    }\n}\nimpl A {\n    #[cfg(test)]\n    fn g(&self) -> bool {}\n}\n";
const FREE_AND_GENERIC: &str = "struct A;\nfn make() -> A {}\nimpl<T> Wrap for T {\n    fn wrapped(self) -> A {}\n}\n";

laws::table!(
    the_transition_graph_of_a_crate_is_measured_with_its_coverage,
    Graphed::shows,
    [
        [CONVERSIONS, "crates/x/src/lib.rs:4: crate x: 2 nodes, 1 edges, 2 transitions, coverage 2 of 2, share 1.000, 0 touch a primitive or unit, 0 touch a foreign or generic type"] => true,
        [PRIMITIVES, "1 nodes, 0 edges, 4 transitions, coverage 0 of 4, share 0.000, 4 touch a primitive or unit, 0 touch"] => true,
        [FOREIGN, "1 nodes, 1 edges, 3 transitions, coverage 0 of 3, share 0.000, 0 touch a primitive or unit, 3 touch a foreign or generic type"] => true,
        [WRAPPED, "2 nodes, 2 edges, 3 transitions, coverage 3 of 3, share 1.000"] => true,
        [TESTED, "crate x"] => false,
        [FREE_AND_GENERIC, "1 nodes, 0 edges, 2 transitions, coverage 1 of 2, share 0.500, 0 touch a primitive or unit, 1 touch a foreign or generic type"] => true,
        ["struct A;\n", "crate x"] => false,
    ]
);

#[test]
fn a_type_resolves_to_its_own_crate_first_and_else_to_the_one_crate_declaring_it() {
    let text = Graphed::of(&[
        Graphed::at(
            "crates/x/src/lib.rs",
            "struct A;\nimpl A {\n    fn b(&self) -> B {}\n    fn c(&self) -> C {}\n}\n",
        ),
        Graphed::at("crates/y/src/lib.rs", "struct B;\nstruct C;\n"),
        Graphed::at("crates/z/src/lib.rs", "struct C;\nstruct A;\nimpl A {\n    fn b(&self) -> B {}\n}\n"),
    ]);
    assert!(text.contains("crate x: 1 nodes, 1 edges, 2 transitions, coverage 1 of 2, share 0.500, 0 touch a primitive or unit, 1 touch a foreign"), "{text}");
    assert!(text.contains("crate z: 2 nodes, 1 edges, 1 transitions, coverage 1 of 1, share 1.000"), "{text}");
}
