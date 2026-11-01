use lint_fixtures::{Located, Source, Sources, Text};
use lint_walk_similarity::WalkSimilarity;

struct Candidates;

impl Candidates {
    fn leaked(text: String) -> &'static str {
        Box::leak(text.into_boxed_str())
    }

    fn lines(text: String) -> Vec<String> {
        let sources = [Source::new(Located::new("a.rs"), Text::new(Self::leaked(text)))];
        Sources::report::<WalkSimilarity>(&sources).violations().iter().map(ToString::to_string).collect()
    }

    fn typed(name: &str, body: &str) -> String {
        format!("struct {name};\nimpl {name} {{\n{body}\n}}\n")
    }

    fn count(pair: &[&str]) -> usize {
        match pair {
            [first, second] => Self::lines(Self::typed("A", first) + &Self::typed("B", second)).len(),
            _ => 0,
        }
    }
}

const FOUR: &str = "fn a(&self) {} fn b(&self) {} fn c(&self) {} fn d(&self) {}";
const FIVE: &str = "fn a(&self) {} fn b(&self) {} fn c(&self) {} fn d(&self) {} fn e(&self) {}";
const SIX: &str = "fn a(&self) {} fn b(&self) {} fn c(&self) {} fn d(&self) {} fn e(&self) {} fn f(&self) {}";
const THREE: &str = "fn a(&self) {} fn b(&self) {} fn c(&self) {}";

laws::table!(
    types_are_flagged_by_how_much_of_their_walk_they_share,
    Candidates::count,
    [
        [FOUR, FOUR] => 1,
        [FIVE, FOUR] => 1,
        [SIX, FOUR] => 0,
        [THREE, THREE] => 0,
        [FOUR, "fn a(&self) {} fn b(&self) {} fn c(&self) {} fn x(&self) {}"] => 0,
        [FOUR, "fn a(&self, x: u8) {} fn b(&self) {} fn c(&self) {} fn d(&self) {}"] => 0,
        ["fn a(&self) -> u8 {} fn b(&self) {} fn c(&self) {} fn d(&self) {}", "fn a(&self) -> u16 {} fn b(&self) {} fn c(&self) {} fn d(&self) {}"] => 0,
        ["fn a(&self) -> A {} fn b(&self) {} fn c(&self) {} fn d(&self) {}", "fn a(&self) -> B {} fn b(&self) {} fn c(&self) {} fn d(&self) {}"] => 1,
        ["fn a(&self) -> A {} fn b(&self) {} fn c(&self) {} fn d(&self) {}", "fn a(&self) -> A {} fn b(&self) {} fn c(&self) {} fn d(&self) {}"] => 0,
    ]
);

#[test]
fn a_finding_names_both_types_where_the_second_is_and_what_they_share() {
    let lines = Candidates::lines(Candidates::typed("A", FOUR) + &Candidates::typed("B", FOUR));
    assert_eq!(lines.len(), 1);
    assert!(
        lines
            .iter()
            .any(|line| line.contains("a.rs:1: A and B (at a.rs:5) share 4 of 4 outgoing methods and trait impls; ")),
        "{lines:?}"
    );
    assert!(
        lines
            .iter()
            .any(|line| line.contains("a standard trait, else a generic over an existing trait, else a derive")),
        "{lines:?}"
    );
}

#[test]
fn trait_impls_count_by_trait_name_and_derived_traits_do_not() {
    let shared = "impl Display for A {}\nimpl Display for B {}\nimpl Plain for A {}\nimpl Plain for B {}\nimpl From<X> for A {}\nimpl From<Y> for B {}\n#[derive(Debug, Clone)]\nstruct C;\n";
    let methods =
        format!("{}{}{shared}", Candidates::typed("A", "fn m(&self) {}"), Candidates::typed("B", "fn m(&self) {}"));
    let lines = Candidates::lines(methods);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert!(lines.iter().any(|line| line.contains("share 4 of 4")), "{lines:?}");
}

#[test]
fn types_deriving_newtype_are_never_candidates() {
    let text =
        format!("#[derive(Newtype)]\n{}struct Other;\n{}", Candidates::typed("A", FOUR), Candidates::typed("B", FOUR));
    assert_eq!(Candidates::lines(text).len(), 0);
    let both = format!(
        "#[derive(Newtype, Clone)]\n{}#[derive(newtype::Newtype)]\n{}",
        Candidates::typed("A", FOUR),
        Candidates::typed("B", FOUR)
    );
    assert_eq!(Candidates::lines(both).len(), 0);
}

#[test]
fn every_alike_pair_is_listed_once() {
    let text = Candidates::typed("A", FOUR) + &Candidates::typed("B", FOUR) + &Candidates::typed("C", FOUR);
    assert_eq!(Candidates::lines(text).len(), 3);
}

#[test]
fn types_in_different_crates_can_be_candidates_and_test_code_is_left_out() {
    let sources = [
        Source::new(Located::new("crates/x/src/lib.rs"), Text::new(Candidates::leaked(Candidates::typed("A", FOUR)))),
        Source::new(Located::new("crates/y/src/lib.rs"), Text::new(Candidates::leaked(Candidates::typed("B", FOUR)))),
        Source::new(
            Located::new("crates/z/src/lib.rs"),
            Text::new(Candidates::leaked(format!("#[cfg(test)]\nmod tests {{\n{}}}\n", Candidates::typed("C", FOUR)))),
        ),
    ];
    let report = Sources::report::<WalkSimilarity>(&sources);
    assert_eq!(report.violations().len(), 1);
}

#[test]
fn a_name_declared_twice_in_a_crate_is_never_a_candidate() {
    let text = Candidates::typed("A", FOUR) + &Candidates::typed("B", FOUR) + "struct A;\n";
    assert_eq!(Candidates::lines(text).len(), 0);
}

const LOCAL_TRAIT: &str = "trait Tr {\n    fn a(&self);\n    fn b(&self);\n    fn c(&self) -> A;\n}\n";

#[test]
fn methods_of_a_local_trait_impl_join_the_walk_but_those_of_a_foreign_trait_do_not() {
    let local = format!(
        "{LOCAL_TRAIT}struct A;\nimpl Tr for A {{\n    fn a(&self) {{}}\n    fn b(&self) {{}}\n    fn c(&self) -> A {{ A }}\n}}\nstruct B;\nimpl Tr for B {{\n    fn a(&self) {{}}\n    fn b(&self) {{}}\n    fn c(&self) -> B {{ B }}\n}}\n"
    );
    assert_eq!(Candidates::lines(local).len(), 1);
    let foreign = "struct A;\nimpl Foreign for A {\n    fn a(&self) {}\n    fn b(&self) {}\n    fn c(&self) {}\n}\nstruct B;\nimpl Foreign for B {\n    fn a(&self) {}\n    fn b(&self) {}\n    fn c(&self) {}\n}\n";
    assert_eq!(Candidates::lines(foreign.to_owned()).len(), 0);
}

#[test]
fn a_provided_method_decides_the_walk_by_its_trait_arity_and_return() {
    let one = "struct A;\nimpl Tr for A {\n    fn a(&self) {}\n    fn b(&self) {}\n    fn c(&self) -> A { A }\n}\nstruct B;\nimpl Tr for B {\n    fn a(&self) {}\n    fn b(&self) {}\n    fn c(&self) -> B { B }\n}\n";
    assert_eq!(Candidates::lines(format!("{LOCAL_TRAIT}{one}")).len(), 1);
    let other = "struct A;\nimpl Tr for A {\n    fn a(&self) {}\n    fn b(&self) {}\n    fn c(&self) -> A { A }\n}\nstruct B;\nimpl Tr for B {\n    fn a(&self) {}\n    fn b(&self, x: u8) {}\n    fn c(&self) -> A { A }\n}\n";
    assert_eq!(Candidates::lines(format!("{LOCAL_TRAIT}{other}")).len(), 0);
}
