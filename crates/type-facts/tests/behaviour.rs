use lint_core::Site;
use lint_fixtures::{Located, Source, Sources, Text};
use lint_layout::Name;
use lint_placement::CrateName;
use type_facts::{Dependency, Facts, GenericImpl, Method, Shape, TypeFact, Via};

struct Described;

impl Described {
    fn file(path: &'static str, text: &'static str) -> Source {
        Source::new(Located::new(path), Text::new(text))
    }

    fn names(facts: &Facts<'_>) -> Vec<String> {
        facts.types().iter().map(|found| AsRef::<Name>::as_ref(found).to_string()).collect()
    }

    fn only<'facts>(facts: &'facts Facts<'facts>) -> &'facts TypeFact<'facts> {
        facts.types().first().unwrap_or_else(|| unreachable!("a type was declared"))
    }

    fn methods(facts: &Facts<'_>, found: &TypeFact<'_>) -> Vec<String> {
        facts
            .methods(found)
            .iter()
            .map(|method| {
                let returns = method.returns().map(ToString::to_string).unwrap_or_default();
                format!("{}/{}->{returns}", AsRef::<Name>::as_ref(method), method.arity())
            })
            .collect()
    }

    fn implemented(facts: &Facts<'_>, found: &TypeFact<'_>) -> Vec<String> {
        facts
            .implementations(found)
            .iter()
            .map(|each| {
                let argument = each.argument().map(ToString::to_string).unwrap_or_default();
                format!("{}<{argument}>", AsRef::<Name>::as_ref(each))
            })
            .collect()
    }
}

#[test]
fn structs_and_enums_are_types_with_their_derives_and_site() {
    let files = Sources::parsed(&[Described::file(
        "a.rs",
        "#[derive(Debug, Clone)]\nstruct A;\n\nenum B { X }\nunion U { x: u8 }\n",
    )]);
    let facts = Facts::of_files(&files);
    assert_eq!(Described::names(&facts), ["A", "B"]);
    let first = Described::only(&facts);
    assert_eq!(first.derives().iter().map(ToString::to_string).collect::<Vec<_>>(), ["Debug", "Clone"]);
    assert_eq!(AsRef::<Site>::as_ref(first).to_string(), "a.rs:2");
}

#[test]
fn derives_through_paths_and_other_attributes_are_read_by_their_last_name() {
    let files = Sources::parsed(&[Described::file(
        "a.rs",
        "#[derive(newtype::Newtype)]\n#[allow(dead_code)]\nstruct A(u8);\n",
    )]);
    let facts = Facts::of_files(&files);
    assert_eq!(Described::only(&facts).derives().iter().map(ToString::to_string).collect::<Vec<_>>(), ["Newtype"]);
}

#[test]
fn inherent_methods_are_kept_with_arity_and_returned_type() {
    let files = Sources::parsed(&[Described::file(
        "a.rs",
        "struct A;\nimpl A {\n    fn one(&self) {}\n    fn two(&self, x: u8) -> Vec<B> { f() }\n    fn three(self) -> &B { f() }\n    fn four() -> (u8, u8) { f() }\n    const C: u8 = 1;\n}\n",
    )]);
    let facts = Facts::of_files(&files);
    assert_eq!(
        Described::methods(&facts, Described::only(&facts)),
        ["one/1->", "two/2->Vec", "three/1->B", "four/0->"]
    );
}

#[test]
fn trait_impls_keep_the_trait_and_its_first_type_argument() {
    let files = Sources::parsed(&[Described::file(
        "a.rs",
        "struct A;\nimpl From<B> for A { fn from(b: B) -> A { f() } }\nimpl std::fmt::Display for A {}\nimpl AsRef<&C> for A {}\nimpl Tr<'a, D> for A {}\nimpl Plain<1> for A {}\n",
    )]);
    let facts = Facts::of_files(&files);
    let found = Described::only(&facts);
    assert_eq!(Described::implemented(&facts, found), ["From<B>", "Display<>", "AsRef<C>", "Tr<D>", "Plain<>"]);
    assert_eq!(Described::methods(&facts, found), Vec::<String>::new());
    let provided: Vec<String> = facts
        .implementations(found)
        .iter()
        .flat_map(|each| each.methods().iter().map(|method| AsRef::<Name>::as_ref(method).to_string()))
        .collect();
    assert_eq!(provided, ["from"]);
}

#[test]
fn impls_attach_across_files_of_one_crate_but_not_across_crates_or_to_foreign_types() {
    let files = Sources::parsed(&[
        Described::file("crates/x/src/lib.rs", "pub struct A;\nmod m;\n"),
        Described::file("crates/x/src/m.rs", "impl A { fn here(&self) {} }\nimpl Foreign { fn lost(&self) {} }\n"),
        Described::file(
            "crates/y/src/lib.rs",
            "impl A { fn there(&self) {} }\npub struct A;\nimpl &A { fn by_ref(&self) {} }\nimpl (A) { fn bare(&self) {} }\n",
        ),
    ]);
    let facts = Facts::of_files(&files);
    let methods: Vec<Vec<String>> = facts.types().iter().map(|found| Described::methods(&facts, found)).collect();
    assert_eq!(methods, [vec!["here/1->"], vec!["there/1->", "by_ref/1->", "bare/1->"]]);
    let crates: Vec<String> = facts.types().iter().map(|found| AsRef::<CrateName>::as_ref(found).to_string()).collect();
    assert_eq!(crates, ["x", "y"]);
}

#[test]
fn local_traits_are_listed_with_their_crate() {
    let files = Sources::parsed(&[Described::file("crates/x/src/lib.rs", "pub trait T { fn m(&self); }\n")]);
    let facts = Facts::of_files(&files);
    let found = facts.traits().first().unwrap_or_else(|| unreachable!("a trait was declared"));
    assert_eq!(AsRef::<Name>::as_ref(found).to_string(), "T");
    assert_eq!(AsRef::<CrateName>::as_ref(found).to_string(), "x");
    assert_eq!(AsRef::<Site>::as_ref(found).to_string(), "crates/x/src/lib.rs:1");
}

#[test]
fn test_code_and_target_files_are_left_out() {
    let files = Sources::parsed(&[
        Described::file(
            "a.rs",
            "struct A;\n#[cfg(test)]\nmod tests { struct B; trait T {} }\n#[cfg(test)]\nstruct C;\n#[cfg(test)]\ntrait U {}\n#[cfg(test)]\nimpl A { fn lost(&self) {} }\n",
        ),
        Described::file("tests/b.rs", "struct D;\n"),
    ]);
    let facts = Facts::of_files(&files);
    assert_eq!(Described::names(&facts), ["A"]);
    assert_eq!(Described::methods(&facts, Described::only(&facts)), Vec::<String>::new());
    assert_eq!(facts.traits().len(), 0);
}

#[test]
fn a_type_name_declared_twice_in_one_crate_is_not_declared_once() {
    let files = Sources::parsed(&[
        Described::file("crates/x/src/lib.rs", "struct A;\nmod m;\nstruct B;\n"),
        Described::file("crates/x/src/m.rs", "struct A;\n"),
        Described::file("crates/y/src/lib.rs", "struct B;\nstruct A;\n"),
    ]);
    let facts = Facts::of_files(&files);
    let once: Vec<String> = facts
        .types()
        .iter()
        .filter_map(|found| facts.declared_once(found))
        .map(|found| format!("{}:{}", AsRef::<CrateName>::as_ref(found), AsRef::<Name>::as_ref(found)))
        .collect();
    assert_eq!(once, ["x:B", "y:B", "y:A"]);
}

#[test]
fn a_tuple_struct_of_one_field_is_a_newtype_shape_and_nothing_else_is() {
    let files = Sources::parsed(&[Described::file(
        "a.rs",
        "struct A(u8);\nstruct B(u8, u8);\nstruct C { x: u8 }\nstruct D;\nenum E { X(u8) }\nstruct F(pub Vec<u8>);\n",
    )]);
    let facts = Facts::of_files(&files);
    let shapes: Vec<Shape> = facts.types().iter().map(TypeFact::shape).collect();
    assert_eq!(shapes, [Shape::OneField, Shape::Other, Shape::Other, Shape::Other, Shape::Other, Shape::OneField]);
}

#[test]
fn types_and_traits_inside_inline_modules_are_found() {
    let files =
        Sources::parsed(&[Described::file("a.rs", "mod m { struct A; trait T {} impl A { fn x(&self) {} } }\n")]);
    let facts = Facts::of_files(&files);
    assert_eq!(Described::names(&facts), ["A"]);
    assert_eq!(facts.traits().len(), 1);
}

#[test]
fn method_shapes_erase_names_but_keep_the_receiver_self_arguments_and_return_shape() {
    let files = Sources::parsed(&[Described::file(
        "a.rs",
        "struct A;\nimpl A {\n    fn one(&self, other: &A, key: Key) -> Option<A> {}\n    fn two(&mut self) -> Vec<B> {}\n    fn three(self) -> Result<Self, E> {}\n    fn four(item: Self) -> impl Iterator<Item = B> {}\n    fn five(&self) -> &[B] {}\n    fn six() -> Self {}\n    fn seven(&self) -> B {}\n    fn eight(&self) -> () {}\n    fn nine(&self) -> (Self) {}\n    fn ten(&self) -> (A, B) {}\n}\n",
    )]);
    let facts = Facts::of_files(&files);
    let shapes: Vec<String> =
        facts.methods(Described::only(&facts)).iter().map(|method| method.shape().to_string()).collect();
    assert_eq!(
        shapes,
        [
            "fn(&self, Self, _) -> Option",
            "fn(&mut self) -> many",
            "fn(self) -> Result",
            "fn(Self) -> many",
            "fn(&self) -> many",
            "fn() -> Self",
            "fn(&self) -> _",
            "fn(&self)",
            "fn(&self) -> Self",
            "fn(&self) -> _",
        ]
    );
}

#[test]
fn trait_methods_are_kept_with_their_shapes_required_or_provided() {
    let files = Sources::parsed(&[Described::file(
        "crates/x/src/lib.rs",
        "trait Index {\n    fn insert(&mut self, item: Item);\n    fn search(&self, query: &Query) -> Vec<Item> { f() }\n    #[cfg(test)]\n    fn hidden(&self);\n}\n",
    )]);
    let facts = Facts::of_files(&files);
    let found = facts.traits().first().unwrap_or_else(|| unreachable!("a trait was declared"));
    let methods: Vec<String> = AsRef::<[Method]>::as_ref(found)
        .iter()
        .map(|method| format!("{} {}", AsRef::<Name>::as_ref(method), method.shape()))
        .collect();
    assert_eq!(methods, ["insert fn(&mut self, _)", "search fn(&self, _) -> many"]);
}

#[test]
fn generic_impls_are_listed_with_their_trait_target_and_site_and_plain_impls_are_not() {
    let files = Sources::parsed(&[Described::file(
        "crates/x/src/lib.rs",
        "struct A;\nimpl A { fn plain(&self) {} }\nimpl<T: Shape> Render for T {\n    fn render(&self) -> Text {}\n}\nimpl<T> Holder<T> {\n    fn get(&self) -> Option<T> {}\n}\n#[cfg(test)]\nimpl<T> Hidden for T {}\n",
    )]);
    let facts = Facts::of_files(&files);
    let found: Vec<String> = facts
        .generic_impls()
        .iter()
        .map(|each| {
            let shapes: Vec<String> =
                AsRef::<[Method]>::as_ref(each).iter().map(|method| method.shape().to_string()).collect();
            format!(
                "{each} in {} at {}: {}",
                AsRef::<CrateName>::as_ref(each),
                AsRef::<Site>::as_ref(each),
                shapes.join(", ")
            )
        })
        .collect();
    assert_eq!(
        found,
        [
            "Render for T in x at crates/x/src/lib.rs:3: fn(&self) -> _",
            "Holder in x at crates/x/src/lib.rs:6: fn(&self) -> Option"
        ]
    );
    assert_eq!(
        facts.generic_impls().first().and_then(GenericImpl::implemented).map(ToString::to_string),
        Some("Render".to_owned())
    );
}

#[test]
fn implementers_of_a_trait_are_found_across_crates() {
    let files = Sources::parsed(&[
        Described::file("crates/x/src/lib.rs", "trait Index {}\nstruct Text;\nimpl Index for Text {}\nstruct Other;\n"),
        Described::file("crates/y/src/lib.rs", "struct Files;\nimpl x::Index for Files {}\nimpl Clone for Files {}\n"),
    ]);
    let facts = Facts::of_files(&files);
    let found: Vec<String> = facts.implementers(&Name::from("Index")).iter().map(ToString::to_string).collect();
    assert_eq!(found, ["Text", "Files"]);
}

#[test]
fn fields_variants_and_bounds_are_dependencies_marked_when_held_behind_a_pointer() {
    let files = Sources::parsed(&[Described::file(
        "a.rs",
        "struct A<T: Into<B>> { b: B, n: Box<Self>, r: &'static C, t: T }\nenum E { X(lint_core::D) }\ntrait Tr: Sup { type Out: Assoc; }\n",
    )]);
    let facts = Facts::of_files(&files);
    let described = |dependencies: &[Dependency<'_>]| -> Vec<String> {
        dependencies
            .iter()
            .map(|dependency| {
                let qualifier = dependency.qualifier().map(ToString::to_string).unwrap_or_default();
                format!(
                    "{qualifier}{}:{}:{:?}:{}",
                    AsRef::<Name>::as_ref(dependency),
                    AsRef::<Via>::as_ref(dependency),
                    dependency.held(),
                    AsRef::<Site>::as_ref(dependency)
                )
            })
            .collect()
    };
    let types: Vec<Vec<String>> = facts.types().iter().map(|found| described(found.dependencies())).collect();
    assert_eq!(
        types,
        [
            vec![
                "B:field:Directly:a.rs:1".to_owned(),
                "Box:field:Directly:a.rs:1".to_owned(),
                "A:field:Indirectly:a.rs:1".to_owned(),
                "C:field:Indirectly:a.rs:1".to_owned(),
                "Into:bound:Directly:a.rs:1".to_owned(),
                "B:bound:Directly:a.rs:1".to_owned(),
            ],
            vec!["lint_coreD:variant:Directly:a.rs:2".to_owned()],
        ]
    );
    let traits: Vec<Vec<String>> = facts.traits().iter().map(|found| described(found.dependencies())).collect();
    assert_eq!(traits, [vec!["Sup:bound:Directly:a.rs:3".to_owned(), "Assoc:bound:Directly:a.rs:3".to_owned()]]);
}
