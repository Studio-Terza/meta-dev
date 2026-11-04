use lint_core::SourceFile;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_layout::Name;
use transition_facts::{Endpoint, Ground, Transition, Transitions};

struct Read;

impl Read {
    fn endpoint(found: &Endpoint) -> String {
        format!("{:?}:{}", found.nature(), AsRef::<Name>::as_ref(found))
    }

    fn signature(found: &Transition) -> String {
        let takes: Vec<String> = found.takes().iter().map(Self::endpoint).collect();
        let fixed = found.context().trait_name().map(ToString::to_string).unwrap_or_default();
        format!(
            "{found}({}) -> {} {:?} {:?} {fixed}",
            takes.join(", "),
            Self::endpoint(found.gives().endpoint()),
            found.function().mutation(),
            AsRef::<Ground>::as_ref(found)
        )
    }

    fn bodies(found: &Transition) -> String {
        format!("{} {:?}", AsRef::<Name>::as_ref(found.function()), found.function().body())
    }

    fn files(files: &[Source], shown: fn(&Transition) -> String) -> Vec<String> {
        let parsed: Vec<SourceFile> = files.iter().map(|source| Sources::file(source.path(), source.text())).collect();
        Transitions::of_files(&parsed).all().map(|found| shown(&found)).collect()
    }

    fn wrappings(found: &Transition) -> String {
        let function = found.function();
        format!("{} {:?} {:?}", AsRef::<Name>::as_ref(function), function.takings(), function.wrapping())
    }

    fn called(found: &Transition) -> String {
        let calls: Vec<String> = found.function().calls().iter().map(ToString::to_string).collect();
        format!("{} {}", AsRef::<Name>::as_ref(found.function()), calls.join(","))
    }

    fn calls(text: &'static str) -> Vec<String> {
        Self::files(&[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(text))], Self::called)
    }

    fn signatures(text: &'static str) -> Vec<String> {
        Self::files(&[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(text))], Self::signature)
    }

    fn wrapped(text: &'static str) -> Vec<String> {
        Self::files(&[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(text))], Self::wrappings)
    }

    fn body(text: &'static str) -> Vec<String> {
        Self::files(&[Source::new(Located::new("crates/x/src/lib.rs"), Text::new(text))], Self::bodies)
    }
}

#[test]
fn endpoints_read_through_references_and_wrappers_to_domain_types_interfaces_and_black_boxes() {
    let found = Read::signatures(
        "struct A;\nimpl A {\n    fn refs(&self, a: &mut A, b: *const A, c: [A; 2], d: &[A], e: (A)) -> () {}\n    fn tuples(self, pair: (A, u8), both: (A, A)) -> ! {}\n    fn wrapped(&self) -> Option<Vec<Box<A>>> {}\n    fn bare(&self) -> Result {}\n    fn named(&self) -> crate::m::B {}\n    fn traits(&self, f: impl Fn(A) -> A, d: &dyn Render, p: fn(A) -> A, i: impl Iterator<Item = A>, j: impl IntoIterator) -> impl Into<A> {}\n    fn generic<T: Clone, U, V: ?Sized + Debug, W: 'static>(&self, t: T, u: U, v: &V, w: W) -> Self where U: Copy {}\n}\nimpl<T: Shape> Holder<T> where T: Clone {\n    fn get(&mut self, item: T) -> T {}\n}\nimpl<T> Holder<T> where T: Clone {\n    fn put(&self, item: T) {}\n}\nimpl<T> Wrap for T {\n    fn wrapped(self) -> Self {}\n}\n",
    );
    assert_eq!(
        found,
        [
            "x::A::refs(Named:A, Named:A, Named:A, Named:A, Named:A, Named:A) -> Boxed:() ThroughParameter Core ",
            "x::A::tuples(Named:A, Primitive:u8, Boxed:_) -> Boxed:_ Absent Core ",
            "x::A::wrapped(Named:A) -> Named:A Absent Core ",
            "x::A::bare(Named:A) -> Boxed:() Absent Core ",
            "x::A::named(Named:A) -> Named:B Absent Core ",
            "x::A::traits(Named:A, Interface:Fn, Interface:Render, Interface:Fn, Named:A, Boxed:_) -> Interface:Into Absent Core ",
            "x::A::generic(Named:A, Interface:Clone, Interface:Copy, Interface:Debug, Interface:Sized) -> Named:A Absent Core ",
            "x::Holder::get(Named:Holder, Interface:Shape) -> Interface:Shape ThroughSelf Core ",
            "x::Holder::put(Named:Holder, Interface:Clone) -> Boxed:() Absent Core ",
            "x::Sized::wrapped(Interface:Sized) -> Interface:Sized Absent Core Wrap",
        ]
    );
}

#[test]
fn transitions_carry_their_ground_their_trait_and_provided_trait_bodies() {
    let found = Read::signatures(
        "trait Shape {\n    fn area(&self) -> Area;\n    fn scaled(&self, by: &Factor) -> Self { self.with(by) }\n    #[cfg(test)]\n    fn skipped(&self) -> Self {}\n}\n#[cfg(test)]\ntrait Hidden {\n    fn skipped(&self) -> Self {}\n}\nstruct S;\nimpl Shape for S {\n    fn area(&self) -> Area {}\n}\nfn free(s: &mut S) -> S {}\nmod ports {\n    impl P {\n        fn read(&mut self) -> S {}\n    }\n}\nmod boundary {\n    fn parse(text: &str) -> u8 {}\n}\nmod store {\n    impl Records {\n        fn insert(&mut self, record: S) {}\n    }\n    mod ports {\n        fn nested() {}\n    }\n}\nfn after() {}\n#[cfg(test)]\nmod tests {\n    fn hidden() {}\n}\nimpl S {\n    #[cfg(test)]\n    fn hidden(&self) {}\n}\n#[cfg(test)]\nimpl S {\n    fn also(&self) {}\n}\n#[cfg(test)]\nfn gone() {}\n",
    );
    assert_eq!(
        found,
        [
            "x::Shape::scaled(Interface:Shape, Named:Factor) -> Interface:Shape Absent Core Shape",
            "x::S::area(Named:S) -> Named:Area Absent Core Shape",
            "x::free(Named:S) -> Named:S ThroughParameter Core ",
            "x::P::read(Named:P) -> Named:S ThroughSelf Ports ",
            "x::parse(Primitive:str) -> Primitive:u8 Absent Boundary ",
            "x::Records::insert(Named:Records, Named:S) -> Boxed:() ThroughSelf Store ",
            "x::nested() -> Boxed:() Absent Ports ",
            "x::after() -> Boxed:() Absent Core ",
        ]
    );
}

#[test]
fn a_file_takes_its_ground_from_its_module_path_and_tests_are_not_read() {
    let found = Read::files(
        &[
            Source::new(Located::new("crates/x/src/ports.rs"), Text::new("fn a() {}\n")),
            Source::new(Located::new("crates/x/src/boundary.rs"), Text::new("fn b() {}\n")),
            Source::new(Located::new("crates/x/src/store/records.rs"), Text::new("fn c() {}\n")),
            Source::new(Located::new("crates/x/src/plain.rs"), Text::new("fn d() {}\n")),
            Source::new(Located::new("crates/x/tests/behaviour.rs"), Text::new("fn e() {}\n")),
        ],
        Read::signature,
    );
    assert_eq!(
        found,
        [
            "x::a() -> Boxed:() Absent Ports ",
            "x::b() -> Boxed:() Absent Boundary ",
            "x::c() -> Boxed:() Absent Store ",
            "x::d() -> Boxed:() Absent Core ",
        ]
    );
}

#[test]
fn field_focus_is_a_derived_lift_and_other_bodies_are_written() {
    let found = Read::body(
        "impl A {\n    fn a(&self) -> &Name { &self.name }\n    fn b(&self) -> Name { self.name.clone() }\n    fn c(&self) -> Name { (self.inner.name) }\n    fn d(&self) -> Name { *self.name }\n    fn e(&self) -> Name { self.0 }\n    fn f(&self) -> Name { self.name.to_string() }\n    fn g(&self) -> Name { self.name.clone(other) }\n    fn h(&self) -> Name { make().name }\n    fn i(&self) -> Name { crate::C.name }\n    fn j(&self) -> Flag { !self.flag }\n    fn k(&self) -> Name { self.name; }\n    fn l(&self) -> Name { let name = self.name; name }\n    fn m(&self) -> Name { self }\n}\n",
    );
    assert_eq!(
        found,
        [
            "a Focused",
            "b Focused",
            "c Focused",
            "d Focused",
            "e Focused",
            "f Written",
            "g Written",
            "h Written",
            "i Written",
            "j Written",
            "k Written",
            "l Written",
            "m Written",
        ]
    );
}

#[test]
fn a_map_through_a_container_of_an_existing_transition_is_a_derived_lift() {
    let found = Read::body(
        "impl A {\n    fn a(&self) -> Vec<B> { self.items.iter().map(B::from).collect() }\n    fn b(&self) -> Vec<B> { items.into_iter().map(|item| item).collect() }\n    fn c(&self) -> Option<B> { self.b.as_ref().map(|b| &b) }\n    fn d(&self) -> Option<B> { self.b.map(|b| (b)) }\n    fn e(&self) -> Option<B> { self.b.map(|b| b.c) }\n    fn f(&self) -> Option<B> { self.b.map(|b| b.c(d)) }\n    fn g(&self) -> Option<B> { self.b.map(|b| C::new(b)) }\n    fn h(&self) -> Option<B> { self.b.map(|b| b.c(1)) }\n    fn i(&self) -> Option<B> { self.b.map(|b| 1.c()) }\n    fn j(&self) -> Option<B> { self.b.map(|b| C::new()) }\n    fn k(&self) -> Option<B> { self.b.map(|b| (f)(b)) }\n    fn l(&self) -> Option<B> { self.b.map(|b| C::D) }\n    fn m(&self) -> Vec<B> { self.items.iter().cloned().collect() }\n    fn n(&self) -> Vec<B> { self.items.iter().filter(f).map(B::from).collect() }\n    fn o(&self) -> Option<B> { self.b.map(1) }\n    fn p(&self) -> Vec<B> { make().iter().map(B::from).collect() }\n    fn q(&self) -> Option<B> { self.b.map(|b| if b { c } else { d }) }\n}\n",
    );
    assert_eq!(
        found,
        [
            "a Mapped",
            "b Mapped",
            "c Mapped",
            "d Mapped",
            "e Mapped",
            "f Mapped",
            "g Mapped",
            "h Written",
            "i Written",
            "j Written",
            "k Written",
            "l Written",
            "m Written",
            "n Written",
            "o Written",
            "p Written",
            "q Written",
        ]
    );
}

#[test]
fn a_match_that_only_routes_each_variant_to_an_existing_transition_is_a_derived_lift() {
    let found = Read::body(
        "impl E {\n    fn a(&self) -> B { match self { E::X(a) => a.b(), E::Y { inner } => { inner.b() } } }\n    fn b(&self) -> B { match self { E::Z(_, (a)) | E::W((a)) => B::from(a) } }\n    fn c(&self) -> B { match (self, other) { (E::X(a), _) => a.b() } }\n    fn d(&self) -> B { match self { &E::X(ref a) => a.b() } }\n    fn e(&self) -> B { match self { E::X(a) if a.ok() => a.b(), E::Y(a) => a.b() } }\n    fn f(&self) -> B { match self { E::X(a) => a.b(), E::Y { .. } => c.b() } }\n    fn g(&self) -> B { match self { E::X(a) => { let y = a; y.b() } } }\n    fn h(&self) -> B { match self { E::X(a) => a.b(1) } }\n}\n",
    );
    assert_eq!(
        found,
        ["a Routed", "b Routed", "c Routed", "d Routed", "e Written", "f Written", "g Written", "h Written"]
    );
}

#[test]
fn a_domain_type_named_output_is_not_the_associated_type() {
    let found = Read::signatures(
        "struct A;\nstruct Output;\nimpl Add for A {\n    type Output = A;\n    fn add(self, other: A) -> Self::Output {}\n    fn make(self) -> Output {}\n    fn viewed(self) -> <A as Add>::Output {}\n}\n",
    );
    assert_eq!(
        found,
        [
            "x::A::add(Named:A, Named:A) -> Named:A Absent Core Add",
            "x::A::make(Named:A) -> Named:Output Absent Core Add",
            "x::A::viewed(Named:A) -> Named:A Absent Core Add",
        ]
    );
}

#[test]
fn a_body_records_the_names_it_calls_by_plain_call_and_by_method_in_the_order_they_run() {
    let found = Read::calls(
        "struct A;\nimpl A {\n    fn plain(self) -> A { helper(self) }\n    fn pathed(self) -> A { module::helper(self) }\n    fn nested(self) -> A { outer(inner(self)) }\n    fn method(self) -> A { self.helper() }\n    fn mixed(self) -> A { first(self).second() }\n}\n",
    );
    assert_eq!(
        found,
        ["plain helper", "pathed module::helper", "nested inner,outer", "method self.helper", "mixed first,#0.second"]
    );
}

#[test]
fn an_iterator_taken_or_given_reads_as_many_like_a_collection() {
    let found = Read::wrapped(
        "struct A;\nimpl A {\n    fn many(&self, items: impl IntoIterator<Item = A>) -> impl Iterator<Item = A> {}\n    fn boxed(&self) -> Box<dyn Iterator<Item = A>> {}\n    fn listed(&self) -> Vec<A> {}\n    fn shown(&self, shape: &dyn Shape) -> impl Display {}\n}\n",
    );
    assert_eq!(
        found,
        ["many [Plain, Many] Many", "boxed [Plain] Many", "listed [Plain] Many", "shown [Plain, Plain] Plain"]
    );
}
