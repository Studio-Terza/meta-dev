use lint_fixtures::{Case, Flagged, Lines, Located, Queried, Source, Sources, Text, Title};
use lint_repeated_reads::RepeatedReads;

const OBJECTS: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { fact(c) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const UNDISPATCHED: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { fact(c) + 2 } }\n";
const ONE_TYPE: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { c.a } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const FIELDS: &str = "struct Inner { b: u8 }\nstruct Ctx { a: Inner }\nfn fact(i: &Inner) -> u8 { i.b }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(&c.a) + 1 } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { fact(&(c.a)) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const CONSTRUCTORS: &str = "struct Ctx { a: u8 }\nstruct Held<'c>(&'c Ctx);\nenum Kept<'c> { One(&'c Ctx) }\ntrait Lint { fn run(&self, c: &Ctx) -> usize; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> usize { [Some(c)].len() + [Held(c)].len() + [Kept::One(c)].len() } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> usize { [Some(c)].len() + [Held(c)].len() + [Kept::One(c)].len() } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const ASSOCIATED: &str = "struct Ctx { a: u8 }\nstruct Facts;\nimpl Facts { fn of(c: &Ctx) -> u8 { c.a } }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { Facts::of(c) + 1 } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { Facts::of(c) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const GENERIC_OWNER: &str = "struct Ctx { a: u8 }\ntrait Fact { fn of(c: &Ctx) -> u8; }\ntrait Lint { fn run<F: Fact>(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run<F: Fact>(&self, c: &Ctx) -> u8 { F::of(c) + 1 } }\nimpl Lint for B { fn run<F: Fact>(&self, c: &Ctx) -> u8 { F::of(c) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const FORWARDS: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c) } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { fact(c) } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const PRIMITIVES: &str = "fn fact(c: u8) -> u8 { c }\ntrait Lint { fn run(&self, c: u8) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: u8) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(&self, c: u8) -> u8 { fact(c) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const TABLE: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(c: &Ctx) -> u8 { fact(c) + 2 } }\nconst RUNS: [fn(&Ctx) -> u8; 2] = [<A as Lint>::run, <B as Lint>::run];\n";
const VALUE_OUTSIDE_TABLE: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(c: &Ctx) -> u8 { fact(c) + 2 } }\nfn pick() -> fn(&Ctx) -> u8 { <A as Lint>::run }\n";
const METHOD_CALLS: &str = "struct Ctx { a: u8 }\nimpl Ctx { fn fact(&self) -> u8 { self.a } }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { c.fact() + 1 } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { c.fact() + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";

const CASES: [Case; 12] = [
    Case::new(
        Title::new("two implementations of a dyn trait read the same fact"),
        &[Source::new(Located::new("a.rs"), Text::new(OBJECTS))],
        Flagged::new(&["a.rs:6"]),
    ),
    Case::new(
        Title::new("a trait never run together"),
        &[Source::new(Located::new("a.rs"), Text::new(UNDISPATCHED))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("one implementation reads the fact"),
        &[Source::new(Located::new("a.rs"), Text::new(ONE_TYPE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("the same field of the parameter, through a reference and parentheses"),
        &[Source::new(Located::new("a.rs"), Text::new(FIELDS))],
        Flagged::new(&["a.rs:7"]),
    ),
    Case::new(
        Title::new("prelude variants, tuple structs and enum variants build values"),
        &[Source::new(Located::new("a.rs"), Text::new(CONSTRUCTORS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("an associated function of a crate type"),
        &[Source::new(Located::new("a.rs"), Text::new(ASSOCIATED))],
        Flagged::new(&["a.rs:7"]),
    ),
    Case::new(
        Title::new("an associated function of a generic parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(GENERIC_OWNER))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("bodies that only forward the parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(FORWARDS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a primitive parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(PRIMITIVES))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("methods listed in a const table"),
        &[Source::new(Located::new("a.rs"), Text::new(TABLE))],
        Flagged::new(&["a.rs:6"]),
    ),
    Case::new(
        Title::new("a method value outside a const or static table"),
        &[Source::new(Located::new("a.rs"), Text::new(VALUE_OUTSIDE_TABLE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("facts computed by method calls on the parameter are not caught"),
        &[Source::new(Located::new("a.rs"), Text::new(METHOD_CALLS))],
        Flagged::new(&[]),
    ),
];

#[test]
fn implementations_run_together_do_not_each_read_the_same_fact() {
    assert_eq!(Sources::mismatches::<Queried<RepeatedReads>>(&CASES), Lines::default());
}
