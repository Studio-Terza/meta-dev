use lint_core::{Site, SourceFile};
use lint_fixtures::{Located, Source, Sources, Text};
use lint_layout::Name;
use lint_placement::CrateName;
use type_facts::{Facts, Shape, TypeFact};

struct Described;

impl Described {
    fn file(path: &'static str, text: &'static str) -> Source {
        Source::new(Located::new(path), Text::new(text))
    }

    fn facts(files: &[Source]) -> Facts {
        let parsed: Vec<SourceFile> = files.iter().map(|source| Sources::file(source.path(), source.text())).collect();
        Facts::of_files(&parsed)
    }

    fn names(facts: &Facts) -> Vec<String> {
        facts.types().iter().map(|found| AsRef::<Name>::as_ref(found).to_string()).collect()
    }

    fn only(facts: &Facts) -> &TypeFact {
        facts.types().first().unwrap_or_else(|| unreachable!("a type was declared"))
    }

    fn methods(found: &TypeFact) -> Vec<String> {
        found
            .methods()
            .iter()
            .map(|method| {
                let returns = method.returns().map(ToString::to_string).unwrap_or_default();
                format!("{}/{}->{returns}", AsRef::<Name>::as_ref(method), method.arity())
            })
            .collect()
    }

    fn implemented(found: &TypeFact) -> Vec<String> {
        found
            .implementations()
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
    let facts = Described::facts(&[Described::file(
        "a.rs",
        "#[derive(Debug, Clone)]\nstruct A;\n\nenum B { X }\nunion U { x: u8 }\n",
    )]);
    assert_eq!(Described::names(&facts), ["A", "B"]);
    let first = Described::only(&facts);
    assert_eq!(first.derives().iter().map(ToString::to_string).collect::<Vec<_>>(), ["Debug", "Clone"]);
    assert_eq!(AsRef::<Site>::as_ref(first).to_string(), "a.rs:2");
}

#[test]
fn derives_through_paths_and_other_attributes_are_read_by_their_last_name() {
    let facts = Described::facts(&[Described::file(
        "a.rs",
        "#[derive(newtype::Newtype)]\n#[allow(dead_code)]\nstruct A(u8);\n",
    )]);
    assert_eq!(Described::only(&facts).derives().iter().map(ToString::to_string).collect::<Vec<_>>(), ["Newtype"]);
}

#[test]
fn inherent_methods_are_kept_with_arity_and_returned_type() {
    let facts = Described::facts(&[Described::file(
        "a.rs",
        "struct A;\nimpl A {\n    fn one(&self) {}\n    fn two(&self, x: u8) -> Vec<B> { f() }\n    fn three(self) -> &B { f() }\n    fn four() -> (u8, u8) { f() }\n    const C: u8 = 1;\n}\n",
    )]);
    assert_eq!(Described::methods(Described::only(&facts)), ["one/1->", "two/2->Vec", "three/1->B", "four/0->"]);
}

#[test]
fn trait_impls_keep_the_trait_and_its_first_type_argument() {
    let facts = Described::facts(&[Described::file(
        "a.rs",
        "struct A;\nimpl From<B> for A { fn from(b: B) -> A { f() } }\nimpl std::fmt::Display for A {}\nimpl AsRef<&C> for A {}\nimpl Tr<'a, D> for A {}\nimpl Plain<1> for A {}\n",
    )]);
    let found = Described::only(&facts);
    assert_eq!(Described::implemented(found), ["From<B>", "Display<>", "AsRef<C>", "Tr<D>", "Plain<>"]);
    assert_eq!(Described::methods(found), Vec::<String>::new());
    let provided: Vec<String> = found
        .implementations()
        .iter()
        .flat_map(|each| each.methods().iter().map(|method| AsRef::<Name>::as_ref(method).to_string()))
        .collect();
    assert_eq!(provided, ["from"]);
}

#[test]
fn impls_attach_across_files_of_one_crate_but_not_across_crates_or_to_foreign_types() {
    let facts = Described::facts(&[
        Described::file("crates/x/src/lib.rs", "pub struct A;\nmod m;\n"),
        Described::file("crates/x/src/m.rs", "impl A { fn here(&self) {} }\nimpl Foreign { fn lost(&self) {} }\n"),
        Described::file(
            "crates/y/src/lib.rs",
            "impl A { fn there(&self) {} }\npub struct A;\nimpl &A { fn by_ref(&self) {} }\nimpl (A) { fn bare(&self) {} }\n",
        ),
    ]);
    let methods: Vec<Vec<String>> = facts.types().iter().map(Described::methods).collect();
    assert_eq!(methods, [vec!["here/1->"], vec!["there/1->", "by_ref/1->", "bare/1->"]]);
    let crates: Vec<String> = facts.types().iter().map(|found| AsRef::<CrateName>::as_ref(found).to_string()).collect();
    assert_eq!(crates, ["x", "y"]);
}

#[test]
fn local_traits_are_listed_with_their_crate() {
    let facts = Described::facts(&[Described::file("crates/x/src/lib.rs", "pub trait T { fn m(&self); }\n")]);
    let found = facts.traits().first().unwrap_or_else(|| unreachable!("a trait was declared"));
    assert_eq!(AsRef::<Name>::as_ref(found).to_string(), "T");
    assert_eq!(AsRef::<CrateName>::as_ref(found).to_string(), "x");
    assert_eq!(AsRef::<Site>::as_ref(found).to_string(), "crates/x/src/lib.rs:1");
}

#[test]
fn test_code_and_target_files_are_left_out() {
    let facts = Described::facts(&[
        Described::file(
            "a.rs",
            "struct A;\n#[cfg(test)]\nmod tests { struct B; trait T {} }\n#[cfg(test)]\nstruct C;\n#[cfg(test)]\ntrait U {}\n#[cfg(test)]\nimpl A { fn lost(&self) {} }\n",
        ),
        Described::file("tests/b.rs", "struct D;\n"),
    ]);
    assert_eq!(Described::names(&facts), ["A"]);
    assert_eq!(Described::methods(Described::only(&facts)), Vec::<String>::new());
    assert_eq!(facts.traits().len(), 0);
}

#[test]
fn a_type_name_declared_twice_in_one_crate_is_not_declared_once() {
    let facts = Described::facts(&[
        Described::file("crates/x/src/lib.rs", "struct A;\nmod m;\nstruct B;\n"),
        Described::file("crates/x/src/m.rs", "struct A;\n"),
        Described::file("crates/y/src/lib.rs", "struct B;\nstruct A;\n"),
    ]);
    let once: Vec<String> = facts
        .types()
        .iter()
        .filter(|found| facts.is_declared_once(found))
        .map(|found| format!("{}:{}", AsRef::<CrateName>::as_ref(found), AsRef::<Name>::as_ref(found)))
        .collect();
    assert_eq!(once, ["x:B", "y:B", "y:A"]);
}

#[test]
fn a_tuple_struct_of_one_field_is_a_newtype_shape_and_nothing_else_is() {
    let facts = Described::facts(&[Described::file(
        "a.rs",
        "struct A(u8);\nstruct B(u8, u8);\nstruct C { x: u8 }\nstruct D;\nenum E { X(u8) }\nstruct F(pub Vec<u8>);\n",
    )]);
    let shapes: Vec<Shape> = facts.types().iter().map(TypeFact::shape).collect();
    assert_eq!(shapes, [Shape::OneField, Shape::Other, Shape::Other, Shape::Other, Shape::Other, Shape::OneField]);
}
