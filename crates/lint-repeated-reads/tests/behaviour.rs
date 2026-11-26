use lint_fixtures::{Case, Flagged, Lines, Located, Queried, Shown, Source, Sources, Text, Title};
use lint_repeated_reads::RepeatedReads;

const SAME_LITERAL: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx, deep: bool) -> u8 { c.a }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c, true) + 1 } }\nstruct B;\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { fact(c, true) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const OTHER_LITERALS: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx, deep: bool) -> u8 { c.a }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c, true) + 1 } }\nstruct B;\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { fact(c, false) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const SAME_EXPRESSION: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx, n: u8) -> u8 { c.a + n }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c, (c.a + 1) * 2) + 1 } }\nstruct B;\nimpl Lint for B { fn run(&self, ctx: &Ctx) -> u8 { fact(ctx, (ctx.a + 1) * 2) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const OTHER_GROUPS: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx, n: u8) -> u8 { c.a + n }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c, (c.a + 1) * 2) + 1 } }\nstruct B;\nimpl Lint for B { fn run(&self, ctx: &Ctx) -> u8 { fact(ctx, (ctx.a - 1) * 2) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const TUPLE_AND_ARRAY: &str = "struct Ctx { a: u8 }\nfn fact<T>(c: &Ctx, t: T) -> u8 { c.a }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c, (c.a, c.a)) + 1 } }\nstruct B;\nimpl Lint for B { fn run(&self, ctx: &Ctx) -> u8 { fact(ctx, [ctx.a, ctx.a]) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const EVERY_OPERATOR: &str = "struct Ctx { a: u8 }\nstruct Limits;\nfn fact(c: &Ctx, n: u8) -> u8 { c.a + n }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c, (-c.a + 1 - 2 * 3 / 4 % 5 ^ 6 & 7 | 8 << 1 >> 1, c.b && c.d || c.e, c.a == 1, c.a < 1, c.a <= 1, c.a != 1, c.a >= 1, c.a > 1, !c.b, *c.r, &(c.a + 1), [c.a][0], c.m(c.a), Limits::f(c.a), (c.a + 1).x)) + 1 } }\nstruct B;\nimpl Lint for B { fn run(&self, ctx: &Ctx) -> u8 { fact(ctx, (-ctx.a + 1 - 2 * 3 / 4 % 5 ^ 6 & 7 | 8 << 1 >> 1, ctx.b && ctx.d || ctx.e, ctx.a == 1, ctx.a < 1, ctx.a <= 1, ctx.a != 1, ctx.a >= 1, ctx.a > 1, !ctx.b, *ctx.r, &(ctx.a + 1), [ctx.a][0], ctx.m(ctx.a), Limits::f(ctx.a), (ctx.a + 1).x)) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const LOCAL_INSIDE: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx, n: u8) -> u8 { c.a + n }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { let n = 1; fact(c, n + c.a) + 1 } }\nstruct B;\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { let n = 2; fact(c, n + c.a) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const LOCAL_ARGUMENT: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx, n: u8) -> u8 { c.a + n }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { let n = 1; fact(c, n) + 1 } }\nstruct B;\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { let n = 1; fact(c, n) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const OWN_CONSTANTS: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx, probe: u8) -> u8 { c.a + probe }\ntrait Lint { const PROBE: u8; fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { const PROBE: u8 = 1; fn run(&self, c: &Ctx) -> u8 { fact(c, Self::PROBE) + 1 } }\nimpl Lint for B { const PROBE: u8 = 2; fn run(&self, c: &Ctx) -> u8 { fact(c, Self::PROBE) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const SHARED_CONSTANT: &str = "struct Ctx { a: u8 }\nstruct Limits;\nimpl Limits { const MOST: u8 = 1; }\nfn fact(c: &Ctx, probe: u8) -> u8 { c.a + probe }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c, Limits::MOST) + 1 } }\nstruct B;\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { fact(c, Limits::MOST) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const OBJECTS: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { fact(c) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const UNDISPATCHED: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { fact(c) + 2 } }\n";
const ONE_TYPE: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { c.a } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const FIELDS: &str = "struct Inner { b: u8 }\nstruct Ctx { a: Inner }\nfn fact(i: &Inner) -> u8 { i.b }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(&c.a) + 1 } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { fact(&(c.a)) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const INDEXED: &str = "struct Inner { b: u8 }\nstruct Ctx(Inner);\nfn fact(i: &Inner) -> u8 { i.b }\ntrait Lint { fn run(&self, p: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, p: &Ctx) -> u8 { fact(&p.0) + 1 } }\nimpl Lint for B { fn run(&self, p: &Ctx) -> u8 { fact(&p.0) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const CONSTRUCTORS: &str = "struct Ctx { a: u8 }\nstruct Held<'c>(&'c Ctx);\nenum Kept<'c> { One(&'c Ctx) }\ntrait Lint { fn run(&self, c: &Ctx) -> usize; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> usize { [Some(c)].len() + [Held(c)].len() + [Kept::One(c)].len() } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> usize { [Some(c)].len() + [Held(c)].len() + [Kept::One(c)].len() } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const ASSOCIATED: &str = "struct Ctx { a: u8 }\nstruct Facts;\nimpl Facts { fn of(c: &Ctx) -> u8 { c.a } }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { Facts::of(c) + 1 } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { Facts::of(c) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const GENERIC_OWNER: &str = "struct Ctx { a: u8 }\ntrait Fact { fn of(c: &Ctx) -> u8; }\ntrait Lint { fn run<F: Fact>(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run<F: Fact>(&self, c: &Ctx) -> u8 { F::of(c) + 1 } }\nimpl Lint for B { fn run<F: Fact>(&self, c: &Ctx) -> u8 { F::of(c) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const FORWARDS: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { fact(c) } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { fact(c) } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const FORWARDS_AFTER_A_METHOD: &str = "struct Ctx { a: u8 }\nimpl Ctx { fn touch(&self) {} }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { c.touch(); fact(c) } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { c.touch(); fact(c) } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const PRIMITIVES: &str = "fn fact(c: u8) -> u8 { c }\ntrait Lint { fn run(&self, c: u8) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: u8) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(&self, c: u8) -> u8 { fact(c) + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const TABLE: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(c: &Ctx) -> u8 { fact(c) + 2 } }\nconst RUNS: [fn(&Ctx) -> u8; 2] = [<A as Lint>::run, <B as Lint>::run];\n";
const VALUE_OUTSIDE_TABLE: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(c: &Ctx) -> u8 { fact(c) + 2 } }\nfn pick() -> fn(&Ctx) -> u8 { <A as Lint>::run }\n";
const BOUNDED_CONST: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { const NAMES: &'static [&'static str]; fn run(c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { const NAMES: &'static [&'static str] = &[]; fn run(c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { const NAMES: &'static [&'static str] = &[]; fn run(c: &Ctx) -> u8 { fact(c) + 2 } }\nstruct Table<P>(P);\nimpl<P: Lint> Table<P> { const NAMES: &'static [&'static str] = P::NAMES; }\n";
const BOUNDED_METHOD: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(c: &Ctx) -> u8 { fact(c) + 2 } }\nstruct Table<P>(P);\nimpl<P: Lint> Table<P> { const RUN: fn(&Ctx) -> u8 = P::run; }\n";
const WHERE_BOUNDED_METHOD: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(c: &Ctx) -> u8 { fact(c) + 2 } }\nstruct Table<P>(P);\nimpl<P> Table<P> where P: Lint { const RUN: fn(&Ctx) -> u8 = P::run; }\n";
const BOUNDED_CALL: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(c: &Ctx) -> u8 { fact(c) + 2 } }\nstruct Table<P>(P);\nimpl<P: Lint> Table<P> { const RUN: u8 = P::run(&Ctx { a: 1 }); }\n";
const VALUE_AFTER_TABLES: &str = "struct Ctx { a: u8 }\nfn fact(c: &Ctx) -> u8 { c.a }\ntrait Lint { fn run(c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(c: &Ctx) -> u8 { fact(c) + 1 } }\nimpl Lint for B { fn run(c: &Ctx) -> u8 { fact(c) + 2 } }\nconst LIMIT: u8 = 1;\nstatic HELD: u8 = 1;\nimpl A { const TOP: u8 = 1; }\nfn pick() -> fn(&Ctx) -> u8 { <A as Lint>::run }\n";
const SINKS: &str = "struct Out { a: u8 }\nfn write(o: &mut Out) { o.a = 1; }\ntrait Lint { fn run(&self, o: &mut Out, p: Out) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, o: &mut Out, mut p: Out) -> u8 { write(o); write(&mut p); 1 } }\nimpl Lint for B { fn run(&self, o: &mut Out, mut p: Out) -> u8 { write(o); write(&mut p); 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";
const METHOD_CALLS: &str = "struct Ctx { a: u8 }\nimpl Ctx { fn fact(&self) -> u8 { self.a } }\ntrait Lint { fn run(&self, c: &Ctx) -> u8; }\nstruct A;\nstruct B;\nimpl Lint for A { fn run(&self, c: &Ctx) -> u8 { c.fact() + 1 } }\nimpl Lint for B { fn run(&self, c: &Ctx) -> u8 { c.fact() + 2 } }\nfn all(lints: &[&dyn Lint]) -> usize { lints.len() }\n";

const CASES: [Case; 30] = [
    Case::new(
        Title::new("the same literal is one fact"),
        &[Source::new(Located::new("a.rs"), Text::new(SAME_LITERAL))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("different literals are two facts"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_LITERALS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("the same expression over the parameter is one fact"),
        &[Source::new(Located::new("a.rs"), Text::new(SAME_EXPRESSION))],
        Flagged::new(&["a.rs:5"]),
    ),
    Case::new(
        Title::new("expressions that differ inside a group are two facts"),
        &[Source::new(Located::new("a.rs"), Text::new(OTHER_GROUPS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a tuple and an array of the same values are two facts"),
        &[Source::new(Located::new("a.rs"), Text::new(TUPLE_AND_ARRAY))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("the same expression over every operator is one fact"),
        &[Source::new(Located::new("a.rs"), Text::new(EVERY_OPERATOR))],
        Flagged::new(&["a.rs:6", "a.rs:6"]),
    ),
    Case::new(
        Title::new("an expression over a local cannot be compared"),
        &[Source::new(Located::new("a.rs"), Text::new(LOCAL_INSIDE))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a local argument cannot be compared"),
        &[Source::new(Located::new("a.rs"), Text::new(LOCAL_ARGUMENT))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("the same callee with different constants is two facts"),
        &[Source::new(Located::new("a.rs"), Text::new(OWN_CONSTANTS))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("the same callee with the same constant is one fact"),
        &[Source::new(Located::new("a.rs"), Text::new(SHARED_CONSTANT))],
        Flagged::new(&["a.rs:7"]),
    ),
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
        Title::new("the same indexed field of a newtype parameter"),
        &[Source::new(Located::new("a.rs"), Text::new(INDEXED))],
        Flagged::new(&["a.rs:7"]),
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
        Title::new("a body that calls a method before its tail call is not a forwarder"),
        &[Source::new(Located::new("a.rs"), Text::new(FORWARDS_AFTER_A_METHOD))],
        Flagged::new(&["a.rs:7"]),
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
    Case::new(
        Title::new("an associated const reached through a bounded parameter in a table"),
        &[Source::new(Located::new("a.rs"), Text::new(BOUNDED_CONST))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a method reached through a bounded parameter in a table"),
        &[Source::new(Located::new("a.rs"), Text::new(BOUNDED_METHOD))],
        Flagged::new(&["a.rs:6"]),
    ),
    Case::new(
        Title::new("a method reached through a where-bounded parameter in a table"),
        &[Source::new(Located::new("a.rs"), Text::new(WHERE_BOUNDED_METHOD))],
        Flagged::new(&["a.rs:6"]),
    ),
    Case::new(
        Title::new("a method called inside a const is a call, not a table entry"),
        &[Source::new(Located::new("a.rs"), Text::new(BOUNDED_CALL))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a method value after const and static items is outside their tables"),
        &[Source::new(Located::new("a.rs"), Text::new(VALUE_AFTER_TABLES))],
        Flagged::new(&[]),
    ),
    Case::new(
        Title::new("a mutable parameter passed on is written, not read"),
        &[Source::new(Located::new("a.rs"), Text::new(SINKS))],
        Flagged::new(&[]),
    ),
];

#[test]
fn implementations_run_together_do_not_each_read_the_same_fact() {
    assert_eq!(Sources::mismatches::<Queried<RepeatedReads>>(&CASES), Lines::default());
}

#[test]
fn a_finding_names_the_trait_method_the_reader_its_fields_and_every_site() {
    assert_eq!(
        Shown::reported::<Queried<RepeatedReads>>(&[FIELDS, "Lint::run"]),
        vec![
            "crates/x/src/lib.rs:7: Lint::run is implemented by types that each read fact on parameter 1.a; at crates/x/src/lib.rs:7 crates/x/src/lib.rs:8; compute it once where the method is called and pass it in"
        ]
    );
}

#[test]
fn a_finding_names_an_associated_reader_by_its_owner() {
    assert_eq!(
        Shown::reported::<Queried<RepeatedReads>>(&[ASSOCIATED, "Lint::run"]),
        vec![
            "crates/x/src/lib.rs:7: Lint::run is implemented by types that each read Facts::of on parameter 1; at crates/x/src/lib.rs:7 crates/x/src/lib.rs:8; compute it once where the method is called and pass it in"
        ]
    );
}
