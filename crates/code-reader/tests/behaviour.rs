use code_model::{
    Entry, Function, Item, ItemKind, Model, Named, Owner, Primitive, Projection, Resolved, Sited, TypeRef,
};
use code_reader::Reader;
use lint_core::{SourceFile, SourceText};
use lint_layout::Name;

const DERIVED: &str = "#[derive(Clone,\n    Debug)]\nstruct A;\n#[derive(PartialEq)]\nenum B {\n    C,\n}\n#[derive(Copy)]\nunion D {\n    e: u8,\n}\n";
const WRITTEN: &str = "mod m {\n    impl Clone for A {}\n}\nfn f() {\n    impl A {}\n}\n";
const MIXED: &str = "#[derive(Debug)]\nstruct A;\nimpl Clone for A {}\n";

const RENAMED: &str = "use std::{clone::Clone as Dup};\nimpl Dup for R {}\nimpl a::Dup for R {}\n";
const ROOTED: &str = "impl core::fmt::Debug for R {}\nimpl alloc::borrow::ToOwned for R {}\nimpl ::std::hash::Hash for R {}\nimpl ::serde::Serialize for R {}\nimpl crate::a::T for R {}\nimpl self::T for R {}\nimpl super::T for R {}\n";
const BOUND: &str = "use core::fmt;\nuse self::m::T as U;\nuse other::Shown;\nimpl fmt::Debug for R {}\nimpl U for R {}\nimpl Shown for R {}\n";
const GROUPED: &str = "use std::{fmt::{Debug, Display as Shown}, hash::*};\nimpl Debug for R {}\nimpl Shown for R {}\nimpl Hasher for R {}\n";
const DECLARED: &str =
    "trait Clone {}\nimpl Clone for R {}\nimpl Copy for R {}\nimpl Display for R {}\nimpl a::Clone for R {}\n";
const SCOPED: &str = "use std::clone::Clone as Dup;\nmod a {\n    impl Dup for R {}\n}\nfn f() {\n    use std::fmt::Debug as Show;\n    impl Dup for R {}\n    impl Show for R {}\n    mod b {\n        impl Show for R {}\n    }\n}\nimpl Show for R {}\n";
const SHADOWED: &str = "use std::clone::Clone;\nfn f() {\n    trait Clone {}\n    impl Clone for R {}\n}\n";
const DERIVES: &str = "#[derive(Clone, serde::Serialize)]\nstruct A;\n";
const STD_KINDS: &str = "impl std::any::Any for R {}\nimpl core::any::Any for R {}\nimpl std::io::Write for R {}\nimpl std::string::String for R {}\nimpl std::fmt::Error for R {}\nimpl std::collections::HashMap for R {}\n";

const GLOB_DECLARED: &str = "trait Clone {}\nmod tests {\n    use super::*;\n    impl Clone for R {}\n}\n";
const GLOB_CRATE: &str = "use crate::shapes::*;\nimpl Hash for R {}\n";
const GLOB_STD: &str =
    "use std::fmt::*;\nuse core::hash::*;\nimpl Debug for R {}\nimpl Hash for R {}\nimpl Clone for R {}\n";
const SELF_IMPORT: &str = "use std::fmt::{self, Write};\nimpl fmt::Debug for R {}\n";
const GLOB_OTHER: &str = "use other::*;\nimpl Clone for R {}\n";
const GLOB_UNLISTED: &str = "use std::collections::*;\nimpl Clone for R {}\n";
const GLOB_SHADOWING: &str = "use std::fmt::*;\nimpl Result for R {}\n";
const GLOB_ENUM: &str = "enum E {\n    A,\n}\nuse E::*;\nimpl Clone for R {}\n";
const GLOB_ITSELF: &str =
    "mod m {\n    pub use std::clone::Clone as Dup;\n}\nuse m::*;\nuse self::m::*;\nimpl Dup for R {}\n";

const GLOB_OPAQUE_FIRST: &str = "use other::*;\nuse std::hash::*;\nimpl Clone for R {}\n";
const GLOB_OPAQUE_LAST: &str = "use std::fmt::*;\nuse other::*;\nimpl Debug for R {}\n";
const GLOB_CONFLICT: &str = "use std::fmt::*;\nuse std::io::*;\nimpl Result for R {}\n";
const GLOB_MODULES_CONFLICT: &str =
    "mod a {\n    pub trait T {}\n}\nmod b {\n    pub trait T {}\n}\nuse a::*;\nuse b::*;\nimpl T for R {}\n";
const GLOB_MODULES_AGREE: &str =
    "mod a {\n    pub trait T {}\n}\nmod b {\n    pub use super::a::T;\n}\nuse a::*;\nuse b::*;\nimpl T for R {}\n";
const GLOB_BENEATH_IMPORT: &str =
    "mod a {\n    pub trait Clone {}\n}\nuse a::*;\nuse std::clone::Clone;\nimpl Clone for R {}\n";
const GLOB_BENEATH_DECLARED: &str = "use std::fmt::*;\ntrait Debug {}\nimpl Debug for R {}\nimpl Display for R {}\n";
const GLOB_THROUGH_GLOB: &str =
    "mod a {\n    pub mod b {\n        pub trait T {}\n    }\n}\nuse a::*;\nuse b::*;\nimpl T for R {}\n";
const GLOB_CARRIES_EXPANSION: &str = "mod hash {\n    array_impl!(1, 2);\n}\npub use hash::*;\nimpl Clone for R {}\n";
const GLOB_CARRIES_EXPANSION_ABOVE: &str =
    "pin_project! {\n    struct Limited {}\n}\nmod tests {\n    use super::*;\n    impl Debug for R {}\n}\n";
const GLOB_CARRIES_EXTERNAL: &str = "mod rejection {\n    pub use axum_core::extract::rejection::*;\n}\nuse crate::rejection::*;\nimpl Clone for R {}\n";
const DECLARED_KINDS: &str = "struct Clone;\nenum Eq {}\nunion Copy {\n    a: u8,\n}\ntype Default = u8;\ntrait Hash = Sized;\nimpl Clone for R {}\nimpl Eq for R {}\nimpl Copy for R {}\nimpl Default for R {}\nimpl Hash for R {}\n";

const MACRO_CALL: &str = "make!();\nimpl Clone for R {}\n";
const MACRO_STATEMENT: &str = "fn f() {\n    make!();\n    impl Clone for R {}\n}\n";
const MACRO_DEFINED: &str = "macro_rules! make {\n    () => {};\n}\nimpl Clone for R {}\n";
const MACRO_DEFINED_CALLED: &str = "macro_rules! make {\n    () => {};\n}\nmake!();\nimpl Clone for R {}\n";
const MACRO_ASSERT: &str = "struct R;\nfn f() {\n    assert!(true);\n    impl Clone for R {}\n}\n";
const MACRO_STD_STATEMENTS: &str =
    "fn f() {\n    println!(\"a\");\n    let v = vec![1];\n    debug_assert_eq!(v, v);\n    impl Clone for R {}\n}\n";
const MACRO_PATHED_STATEMENT: &str = "fn f() {\n    std::println!(\"a\");\n    impl Clone for R {}\n}\n";
const BLOCK_CHAIN: &str = "fn f() {\n    use std::fmt::Debug as Show;\n    {\n        impl Show for R {}\n    }\n    mod m {\n        impl Show for R {}\n    }\n}\n";
const BLOCK_SUPER: &str = "use std::clone::Clone as Dup;\nfn f() {\n    use std::fmt::Debug as Dup;\n    mod m {\n        impl super::Dup for R {}\n    }\n}\n";
const CFG_ALTERNATIVES: &str = "#[cfg(unix)]\ntrait Clone {}\n#[cfg(windows)]\ntrait Clone {}\nimpl Clone for R {}\n";
const ALTERNATIVE_IMPORTS: &str =
    "#[cfg(unix)]\nuse std::clone::Clone as Dup;\n#[cfg(windows)]\nuse std::fmt::Debug as Dup;\nimpl Dup for R {}\n";
const DISCARDED: &str =
    "use std::fmt::Debug as _;\nuse std::fmt::{self as shown};\nimpl Debug for R {}\nimpl shown::Debug for R {}\n";
const ITEM_PATH: &str = "struct S;\nimpl S::T for R {}\nimpl Option::T for R {}\nimpl ::a::T for R {}\n";

const BLOCK_GLOB_FALLS_THROUGH: &str =
    "trait Clone {}\nmod a {}\nfn f() {\n    use crate::a::*;\n    use Clone as Dup;\n    impl Dup for R {}\n}\n";
const BLOCK_GLOB_SUPPLIES: &str = "struct Clone;\nmod a {\n    pub trait Clone {}\n}\nfn f() {\n    use crate::a::*;\n    use Clone as Dup;\n    impl Dup for R {}\n}\n";

const OWN_ITEM_HIDES_GLOB: &str = "mod inner {\n    pub trait Clone {}\n}\nmod m {\n    pub use crate::inner::*;\n    trait Clone {}\n}\nuse m::*;\nimpl Clone for R {}\nimpl m::Clone for R {}\n";
const OWN_IMPORT_HIDES_GLOB: &str = "mod inner {\n    pub trait Clone {}\n}\nmod m {\n    pub use crate::inner::*;\n    use std::fmt::Debug as Clone;\n}\nuse m::*;\nimpl Clone for R {}\n";
const OWN_VISIBLE_ITEM_HIDES_GLOB: &str = "mod inner {\n    pub struct Clone;\n}\nmod m {\n    pub use crate::inner::*;\n    pub trait Clone {}\n}\nuse m::*;\nimpl Clone for R {}\n";

const GLOB_PRIVATE_SELF: &str = "trait Clone {}\nmod a {\n    pub(self) use std::clone::Clone;\n}\nfn f() {\n    use crate::a::*;\n    impl Clone for R {}\n}\n";
const GLOB_VISIBLE_CRATE: &str = "mod a {\n    pub(crate) use std::fmt::Debug;\n}\nuse a::*;\nimpl Debug for R {}\n";
const GLOB_RESTRICTED: &str = "mod a {\n    pub(super) use std::fmt::Debug;\n}\nuse a::*;\nimpl Debug for R {}\n";
const GLOB_RESTRICTED_IN: &str = "mod a {\n    pub(in crate) use std::fmt::Debug;\n}\nuse a::*;\nimpl Debug for R {}\n";
const PATH_RESTRICTED: &str = "mod a {\n    pub(super) struct T;\n}\nimpl a::T for R {}\n";
const IMPORT_SEES_ITS_MODULE: &str =
    "mod m {\n    mod hidden {\n        pub trait T {}\n    }\n    pub use hidden::T as X;\n}\nimpl m::X for R {}\n";

const EXTERN_SELF: &str =
    "extern crate self as core;\nmod clone {\n    pub trait Clone {}\n}\nimpl core::clone::Clone for R {}\n";
const EXTERN_SELF_NESTED: &str = "extern crate self as core;\nmod clone {\n    pub trait Clone {}\n}\nmod m {\n    impl core::clone::Clone for R {}\n}\n";
const EXTERN_CORE: &str =
    "extern crate core as c;\nextern crate alloc;\nimpl c::clone::Clone for R {}\nimpl alloc::fmt::Debug for R {}\n";
const EXTERN_OTHER: &str =
    "extern crate serde;\nextern crate serde as s;\nimpl serde::Serialize for R {}\nimpl s::Serialize for R {}\n";
const EXTERN_DISCARDED: &str = "extern crate serde as _;\nimpl Clone for R {}\n";

const DERIVE_SHADOWED: &str = "trait Clone {}\n#[derive(Clone, Debug)]\nstruct A;\n";
const DERIVE_IMPORTED: &str =
    "use serde::Serialize;\nuse std::fmt::Debug as Clone;\n#[derive(Serialize, Clone, Hash)]\nstruct A;\n";
const DERIVE_UNLISTED: &str = "#[derive(Serialize)]\nstruct A;\n";
const DERIVE_GLOBBED: &str = "use std::hash::*;\nuse other::*;\n#[derive(Hash, Debug)]\nstruct A;\n";

const RENAMED_ACROSS: [Fixture; 2] = [
    Fixture::new("src/lib.rs", "pub use std::clone::Clone as Dup;\nmod x;\n"),
    Fixture::new("src/x.rs", "use crate::Dup;\nimpl Dup for R {}\n"),
];
const PARENT_GLOB: [Fixture; 2] = [
    Fixture::new("src/lib.rs", "mod x;\nstruct R;\n"),
    Fixture::new("src/x.rs", "use super::*;\nimpl Clone for R {}\n"),
];
const SIBLING_GLOB: [Fixture; 3] = [
    Fixture::new("src/lib.rs", "mod shapes;\nmod order;\n"),
    Fixture::new("src/shapes.rs", "pub trait Shape {}\n"),
    Fixture::new("src/order.rs", "use super::shapes::*;\nimpl PartialEq for R {}\n"),
];
const PARENT_DECLARES: [Fixture; 2] = [
    Fixture::new("src/lib.rs", "trait Clone {}\nmod x;\n"),
    Fixture::new("src/x.rs", "use super::*;\nimpl Clone for R {}\n"),
];
const PRIVATE_IMPORT: [Fixture; 4] = [
    Fixture::new("src/lib.rs", "mod p;\nmod q;\n"),
    Fixture::new("src/p.rs", "use std::hash::Hash;\nmod child;\n"),
    Fixture::new("src/p/child.rs", "use super::*;\nimpl Hash for R {}\n"),
    Fixture::new("src/q.rs", "use crate::p::*;\nimpl Hash for R {}\n"),
];
const REEXPORTED: [Fixture; 4] = [
    Fixture::new("src/lib.rs", "mod a;\nmod b;\nmod c;\n"),
    Fixture::new("src/a.rs", "pub use std::fmt::Debug as Shown;\n"),
    Fixture::new("src/b.rs", "pub use crate::a::Shown;\nuse crate::a::Shown as Hidden;\n"),
    Fixture::new("src/c.rs", "use crate::b::Shown;\nimpl Shown for R {}\nimpl crate::b::Hidden for R {}\n"),
];
const CYCLE: [Fixture; 3] = [
    Fixture::new("src/lib.rs", "mod a;\nmod b;\n"),
    Fixture::new("src/a.rs", "pub use crate::b::X;\n"),
    Fixture::new("src/b.rs", "pub use crate::a::X;\nimpl X for R {}\n"),
];
const RE_EXPORT_CYCLE: [Fixture; 11] = [
    Fixture::new(
        "src/lib.rs",
        "pub trait Mark {}\nmod m0;\nmod m1;\nmod m2;\nmod m3;\nmod m4;\nmod m5;\nmod m6;\nmod m7;\nmod m8;\nmod m9;\npub use m0::*;\npub use m1::*;\npub use m2::*;\npub use m3::*;\npub use m4::*;\npub use m5::*;\npub use m6::*;\npub use m7::*;\npub use m8::*;\npub use m9::*;\n",
    ),
    Fixture::new("src/m0.rs", "pub use super::*;\n"),
    Fixture::new("src/m1.rs", "pub use super::*;\n"),
    Fixture::new("src/m2.rs", "pub use super::*;\n"),
    Fixture::new("src/m3.rs", "pub use super::*;\n"),
    Fixture::new("src/m4.rs", "pub use super::*;\n"),
    Fixture::new("src/m5.rs", "pub use super::*;\n"),
    Fixture::new("src/m6.rs", "pub use super::*;\n"),
    Fixture::new("src/m7.rs", "pub use super::*;\n"),
    Fixture::new("src/m8.rs", "pub use super::*;\n"),
    Fixture::new("src/m9.rs", "pub use super::*;\nimpl Mark for R {}\nimpl Clone for R {}\n"),
];
const WORKSPACE: [Fixture; 2] = [
    Fixture::new("crates/a/src/lib.rs", "pub use std::clone::Clone as Dup;\n"),
    Fixture::new("crates/b/src/lib.rs", "impl a::Dup for R {}\nimpl ::a::Dup for R {}\n"),
];
const ISLAND: [Fixture; 1] = [Fixture::new("src/x.rs", "trait T {}\nimpl crate::T for R {}\nimpl super::T for R {}\n")];
const ISLAND_OWN: [Fixture; 1] = [Fixture::new(
    "src/x.rs",
    "trait T {}\nmod m {\n    pub trait U {}\n}\nimpl T for R {}\nimpl self::T for R {}\nimpl m::U for R {}\n",
)];
const ISLAND_BESIDE_ROOT: [Fixture; 2] = [
    Fixture::new("src/lib.rs", "pub trait T {}\nimpl T for R {}\n"),
    Fixture::new("src/x.rs", "trait T {}\nimpl T for R {}\n"),
];
const PATHED: [Fixture; 2] = [
    Fixture::new("src/lib.rs", "#[path = \"y.rs\"]\nmod x;\nimpl x::T for R {}\n"),
    Fixture::new("src/x.rs", "pub trait T {}\n"),
];
const NESTED_FILES: [Fixture; 5] = [
    Fixture::new("src/lib.rs", "pub trait Mark {}\nmod a;\nmod c;\n"),
    Fixture::new("src/a/mod.rs", "mod b;\n"),
    Fixture::new("src/a/b.rs", "impl super::super::Mark for R {}\n"),
    Fixture::new("src/c.rs", "mod d {\n    mod e;\n}\n"),
    Fixture::new("src/c/d/e.rs", "impl crate::Mark for R {}\n"),
];
const TWO_CANDIDATES: [Fixture; 3] = [
    Fixture::new("src/lib.rs", "mod x;\nimpl x::T for R {}\n"),
    Fixture::new("src/x.rs", "pub trait T {}\n"),
    Fixture::new("src/x/mod.rs", "pub trait T {}\n"),
];

struct Fixture {
    path: &'static str,
    text: &'static str,
}

impl Fixture {
    const fn new(path: &'static str, text: &'static str) -> Fixture {
        Fixture { path, text }
    }
}

#[derive(Debug, PartialEq)]
enum Seen {
    Resolved(Resolved),
    Item(Item),
}

impl Seen {
    fn of(model: &Model, resolved: Resolved) -> Seen {
        match resolved.crate_item().and_then(|id| model.declared(id)) {
            Some(item) => Seen::Item(item.clone()),
            None => Seen::Resolved(resolved),
        }
    }

    fn typed(model: &Model, ty: TypeRef) -> TypeRef<Seen> {
        ty.mapped(&mut |sited: Sited| Seen::of(model, AsRef::<Resolved>::as_ref(&sited).clone()))
    }
}

#[derive(Debug, PartialEq)]
struct Formed {
    name: Name,
    owner: Owner,
    entry: Entry,
    output: Option<TypeRef<Seen>>,
}

impl Formed {
    fn of(model: &Model, function: &Function) -> Formed {
        Formed {
            name: AsRef::<Name>::as_ref(function).clone(),
            owner: function.owner(),
            entry: function.entry(),
            output: Read::written(function).map(|ty| Seen::typed(model, ty)),
        }
    }
}

#[derive(Debug, PartialEq)]
struct Sighted {
    subject: Name,
    implemented: Option<Seen>,
}

struct Expect;

impl Expect {
    fn written(subject: &str, implemented: Seen) -> Sighted {
        Sighted { subject: Name::from(subject), implemented: Some(implemented) }
    }

    fn std(path: &[&str]) -> Seen {
        Seen::Resolved(Resolved::Std(path.iter().map(|segment| Name::from(*segment)).collect()))
    }

    fn std_trait(path: &[&str]) -> Seen {
        Seen::Resolved(Resolved::StdTrait(path.iter().map(|segment| Name::from(*segment)).collect()))
    }

    fn krate(path: &[&str]) -> Seen {
        Expect::item(path.last().copied().unwrap_or_default(), ItemKind::Trait)
    }

    fn item(name: &str, kind: ItemKind) -> Seen {
        Seen::Item(Item::new(Name::from(name), kind))
    }

    fn external(path: &[&str]) -> Seen {
        Seen::Resolved(Resolved::External(path.iter().map(|segment| Name::from(*segment)).collect()))
    }

    fn unknown(path: &[&str]) -> Seen {
        Seen::Resolved(Resolved::Unknown(path.iter().map(|segment| Name::from(*segment)).collect()))
    }

    fn named(path: Seen, arguments: Vec<TypeRef<Seen>>) -> TypeRef<Seen> {
        TypeRef::Named(Named::new(path, arguments))
    }

    fn primitive(primitive: Primitive) -> TypeRef<Seen> {
        Expect::named(Seen::Resolved(Resolved::Primitive(primitive)), Vec::new())
    }

    fn param(name: &str) -> TypeRef<Seen> {
        Expect::named(Seen::Resolved(Resolved::Param(Name::from(name))), Vec::new())
    }

    fn function(name: &str, owner: Owner, entry: Entry, output: Option<TypeRef<Seen>>) -> Formed {
        Formed { name: Name::from(name), owner, entry, output }
    }

    fn projected(subject: TypeRef<Seen>) -> TypeRef<Seen> {
        TypeRef::Projected(Box::new(Projection::new(subject, None)))
    }
}

struct Read;

impl Read {
    fn file(path: &str, text: &str) -> SourceFile {
        match SourceFile::parse(path, SourceText::new(text.to_owned())) {
            Ok(file) => file,
            Err(failure) => unreachable!("the source parses: {failure}"),
        }
    }

    fn sites(files: &[SourceFile]) -> Vec<String> {
        Reader::read_files(files).implementations().map(|found| found.site().to_string()).collect()
    }

    fn implementations(files: &[SourceFile]) -> Vec<Sighted> {
        let model = Reader::read_files(files);
        model
            .implementations()
            .map(|found| Sighted {
                subject: found.entity().subject().clone(),
                implemented: found.entity().implemented().cloned().map(|resolved| Seen::of(&model, resolved)),
            })
            .collect()
    }

    fn resolved(text: &str) -> Vec<Seen> {
        Read::implemented(&[Read::file("a.rs", text)])
    }

    fn across(fixtures: &[Fixture]) -> Vec<Seen> {
        let files: Vec<SourceFile> = fixtures.iter().map(|fixture| Read::file(fixture.path, fixture.text)).collect();
        Read::implemented(&files)
    }

    fn implemented(files: &[SourceFile]) -> Vec<Seen> {
        let model = Reader::read_files(files);
        model
            .implementations()
            .filter_map(|found| found.entity().implemented().cloned())
            .map(|resolved| Seen::of(&model, resolved))
            .collect()
    }

    fn subjects(text: &str) -> Vec<TypeRef<Seen>> {
        let model = Reader::read_files(&[Read::file("a.rs", text)]);
        model.implementations().map(|found| Seen::typed(&model, found.entity().subject_type().clone())).collect()
    }

    fn entries(fixtures: &[Fixture]) -> Vec<Formed> {
        let files: Vec<SourceFile> = fixtures.iter().map(|fixture| Read::file(fixture.path, fixture.text)).collect();
        let model = Reader::read_files(&files);
        model.functions().map(|found| Formed::of(&model, found.entity())).collect()
    }

    fn functions(text: &str) -> Vec<Formed> {
        let model = Reader::read_files(&[Read::file("a.rs", text)]);
        model.functions().map(|found| Formed::of(&model, found.entity())).collect()
    }

    fn outputs(text: &str) -> Vec<TypeRef<Seen>> {
        let model = Reader::read_files(&[Read::file("a.rs", text)]);
        model.functions().filter_map(|found| Read::written(found.entity())).map(|ty| Seen::typed(&model, ty)).collect()
    }

    fn expanded(text: &str) -> Vec<TypeRef<Seen>> {
        let model = Reader::read_files(&[Read::file("a.rs", text)]);
        model
            .functions()
            .filter_map(|found| Read::written(found.entity()))
            .map(|ty| Seen::typed(&model, model.expanded(&ty)))
            .collect()
    }

    fn written(function: &Function) -> Option<TypeRef> {
        function.signature().output().written().cloned()
    }

    fn written_sites(files: &[SourceFile]) -> Vec<String> {
        Reader::read_files(files)
            .implementations()
            .filter_map(|found| found.entity().written().map(|_| found.site().to_string()))
            .collect()
    }
}

laws::table!(every_implementation_is_read_at_its_impl_or_derive_path, |text: &&str| Read::sites(&[Read::file("a.rs", text)]), [
    DERIVED => ["a.rs:1", "a.rs:2", "a.rs:4", "a.rs:8"],
    WRITTEN => ["a.rs:2", "a.rs:5"],
    MIXED => ["a.rs:1", "a.rs:3"],
]);

laws::table!(only_impl_blocks_are_written, |text: &&str| Read::written_sites(&[Read::file("a.rs", text)]), [
    DERIVED => Vec::<&str>::new(),
    WRITTEN => ["a.rs:2", "a.rs:5"],
    MIXED => ["a.rs:3"],
]);

#[test]
fn each_implementation_is_placed_in_its_own_file() {
    assert_eq!(
        Read::sites(&[Read::file("a.rs", MIXED), Read::file("b.rs", WRITTEN)]),
        ["a.rs:1", "a.rs:3", "b.rs:2", "b.rs:5"]
    );
}

#[test]
fn implementations_keep_the_order_of_the_files_read() {
    assert_eq!(
        Read::sites(&[
            Read::file("src/lib.rs", "mod b;\nmod a;\n"),
            Read::file("src/a.rs", "impl Clone for A {}\n"),
            Read::file("src/b.rs", "impl Clone for B {}\n"),
        ]),
        ["src/a.rs:1", "src/b.rs:1"]
    );
}

#[test]
fn a_renamed_trait_is_read_as_its_original() {
    assert_eq!(
        Read::implementations(&[Read::file("a.rs", RENAMED)]),
        [
            Expect::written("R", Expect::std_trait(&["std", "clone", "Clone"])),
            Expect::written("R", Expect::external(&["a", "Dup"])),
        ]
    );
}

#[test]
fn a_std_path_resolves_as_a_trait_only_when_its_item_is_a_trait() {
    assert_eq!(
        Read::resolved(STD_KINDS),
        [
            Expect::std_trait(&["std", "any", "Any"]),
            Expect::std_trait(&["std", "any", "Any"]),
            Expect::std_trait(&["std", "io", "Write"]),
            Expect::std(&["std", "string", "String"]),
            Expect::std(&["std", "fmt", "Error"]),
            Expect::std(&["std", "collections", "HashMap"]),
        ]
    );
}

laws::table!(each_trait_path_resolves_through_the_scopes_it_is_written_in, |text: &&str| Read::resolved(text), [
    ROOTED => [
        Expect::std_trait(&["std", "fmt", "Debug"]),
        Expect::std_trait(&["std", "borrow", "ToOwned"]),
        Expect::std_trait(&["std", "hash", "Hash"]),
        Expect::external(&["serde", "Serialize"]),
        Expect::unknown(&["crate", "a", "T"]),
        Expect::unknown(&["self", "T"]),
        Expect::unknown(&["super", "T"]),
    ],
    BOUND => [
        Expect::std_trait(&["std", "fmt", "Debug"]),
        Expect::unknown(&["U"]),
        Expect::external(&["other", "Shown"]),
    ],
    GROUPED => [
        Expect::std_trait(&["std", "fmt", "Debug"]),
        Expect::std_trait(&["std", "fmt", "Display"]),
        Expect::std_trait(&["std", "hash", "Hasher"]),
    ],
    DECLARED => [
        Expect::krate(&["crate", "Clone"]),
        Expect::std_trait(&["std", "marker", "Copy"]),
        Expect::unknown(&["Display"]),
        Expect::external(&["a", "Clone"]),
    ],
    SCOPED => [
        Expect::unknown(&["Dup"]),
        Expect::std_trait(&["std", "clone", "Clone"]),
        Expect::std_trait(&["std", "fmt", "Debug"]),
        Expect::unknown(&["Show"]),
        Expect::unknown(&["Show"]),
    ],
    SHADOWED => [Expect::krate(&["crate", "Clone"])],
    DERIVES => [Expect::std_trait(&["std", "clone", "Clone"]), Expect::external(&["serde", "Serialize"])],
    BLOCK_CHAIN => [Expect::std_trait(&["std", "fmt", "Debug"]), Expect::unknown(&["Show"])],
    BLOCK_SUPER => [Expect::std_trait(&["std", "clone", "Clone"])],
    CFG_ALTERNATIVES => [Expect::unknown(&["Clone"])],
    DECLARED_KINDS => [
        Expect::item("Clone", ItemKind::Struct),
        Expect::item("Eq", ItemKind::Enum),
        Expect::item("Copy", ItemKind::Union),
        Expect::item("Default", ItemKind::Alias),
        Expect::item("Hash", ItemKind::Trait),
    ],
    ALTERNATIVE_IMPORTS => [Expect::unknown(&["Dup"])],
    DISCARDED => [Expect::unknown(&["Debug"]), Expect::std_trait(&["std", "fmt", "Debug"])],
    ITEM_PATH => [Expect::unknown(&["S", "T"]), Expect::external(&["Option", "T"]), Expect::external(&["a", "T"])],
    IMPORT_SEES_ITS_MODULE => [Expect::krate(&["crate", "m", "hidden", "T"])],
]);

laws::table!(a_glob_import_decides_what_a_bare_trait_name_means, |text: &&str| Read::resolved(text), [
    GLOB_DECLARED => [Expect::krate(&["crate", "Clone"])],
    GLOB_CRATE => [Expect::unknown(&["Hash"])],
    GLOB_STD => [
        Expect::std_trait(&["std", "fmt", "Debug"]),
        Expect::std_trait(&["std", "hash", "Hash"]),
        Expect::std_trait(&["std", "clone", "Clone"]),
    ],
    SELF_IMPORT => [Expect::std_trait(&["std", "fmt", "Debug"])],
    GLOB_OTHER => [Expect::unknown(&["Clone"])],
    GLOB_UNLISTED => [Expect::unknown(&["Clone"])],
    GLOB_SHADOWING => [Expect::std(&["std", "fmt", "Result"])],
    GLOB_ENUM => [Expect::unknown(&["Clone"])],
    GLOB_ITSELF => [Expect::std_trait(&["std", "clone", "Clone"])],
    GLOB_OPAQUE_FIRST => [Expect::unknown(&["Clone"])],
    GLOB_OPAQUE_LAST => [Expect::unknown(&["Debug"])],
    GLOB_CONFLICT => [Expect::unknown(&["Result"])],
    GLOB_MODULES_CONFLICT => [Expect::unknown(&["T"])],
    GLOB_MODULES_AGREE => [Expect::krate(&["crate", "a", "T"])],
    GLOB_BENEATH_IMPORT => [Expect::std_trait(&["std", "clone", "Clone"])],
    GLOB_BENEATH_DECLARED => [Expect::krate(&["crate", "Debug"]), Expect::std_trait(&["std", "fmt", "Display"])],
    GLOB_THROUGH_GLOB => [Expect::krate(&["crate", "a", "b", "T"])],
    GLOB_PRIVATE_SELF => [Expect::krate(&["crate", "Clone"])],
    GLOB_VISIBLE_CRATE => [Expect::std_trait(&["std", "fmt", "Debug"])],
    GLOB_RESTRICTED => [Expect::unknown(&["Debug"])],
    GLOB_RESTRICTED_IN => [Expect::unknown(&["Debug"])],
    PATH_RESTRICTED => [Expect::item("T", ItemKind::Struct)],
]);

laws::table!(a_block_looks_past_its_glob_to_the_enclosing_module, |text: &&str| Read::resolved(text), [
    BLOCK_GLOB_FALLS_THROUGH => [Expect::krate(&["crate", "Clone"])],
    BLOCK_GLOB_SUPPLIES => [Expect::krate(&["crate", "a", "Clone"])],
]);

laws::table!(a_module_s_own_name_hides_the_same_name_in_its_globs, |text: &&str| Read::resolved(text), [
    OWN_ITEM_HIDES_GLOB => [Expect::std_trait(&["std", "clone", "Clone"]), Expect::unknown(&["m", "Clone"])],
    OWN_IMPORT_HIDES_GLOB => [Expect::std_trait(&["std", "clone", "Clone"])],
    OWN_VISIBLE_ITEM_HIDES_GLOB => [Expect::krate(&["crate", "m", "Clone"])],
]);

laws::table!(a_glob_carries_the_hidden_names_of_its_module, |text: &&str| Read::resolved(text), [
    GLOB_CARRIES_EXPANSION => [Expect::unknown(&["Clone"])],
    GLOB_CARRIES_EXPANSION_ABOVE => [Expect::unknown(&["Debug"])],
    GLOB_CARRIES_EXTERNAL => [Expect::unknown(&["Clone"])],
]);

laws::table!(an_extern_crate_item_binds_its_name, |text: &&str| Read::resolved(text), [
    EXTERN_SELF => [Expect::krate(&["crate", "clone", "Clone"])],
    EXTERN_SELF_NESTED => [Expect::krate(&["crate", "clone", "Clone"])],
    EXTERN_CORE => [Expect::std_trait(&["std", "clone", "Clone"]), Expect::std_trait(&["std", "fmt", "Debug"])],
    EXTERN_OTHER => [Expect::external(&["serde", "Serialize"]), Expect::external(&["serde", "Serialize"])],
    EXTERN_DISCARDED => [Expect::std_trait(&["std", "clone", "Clone"])],
]);

laws::table!(a_derive_resolves_in_the_macro_namespace, |text: &&str| Read::resolved(text), [
    DERIVE_SHADOWED => [Expect::std_trait(&["std", "clone", "Clone"]), Expect::std_trait(&["std", "fmt", "Debug"])],
    DERIVE_IMPORTED => [
        Expect::external(&["serde", "Serialize"]),
        Expect::std_trait(&["std", "fmt", "Debug"]),
        Expect::std_trait(&["std", "hash", "Hash"]),
    ],
    DERIVE_UNLISTED => [Expect::unknown(&["Serialize"])],
    DERIVE_GLOBBED => [Expect::unknown(&["Hash"]), Expect::unknown(&["Debug"])],
]);

laws::table!(an_item_position_macro_hides_what_a_bare_name_means, |text: &&str| Read::resolved(text), [
    MACRO_CALL => [Expect::unknown(&["Clone"])],
    MACRO_STATEMENT => [Expect::unknown(&["Clone"])],
    MACRO_DEFINED => [Expect::std_trait(&["std", "clone", "Clone"])],
    MACRO_DEFINED_CALLED => [Expect::unknown(&["Clone"])],
    MACRO_PATHED_STATEMENT => [Expect::unknown(&["Clone"])],
]);

laws::table!(a_std_statement_macro_hides_nothing, |text: &&str| Read::resolved(text), [
    MACRO_ASSERT => [Expect::std_trait(&["std", "clone", "Clone"])],
    MACRO_STD_STATEMENTS => [Expect::std_trait(&["std", "clone", "Clone"])],
]);

laws::table!(a_path_resolves_across_the_files_of_its_crate, Read::across, [
    RENAMED_ACROSS => [Expect::std_trait(&["std", "clone", "Clone"])],
    PARENT_GLOB => [Expect::std_trait(&["std", "clone", "Clone"])],
    SIBLING_GLOB => [Expect::std_trait(&["std", "cmp", "PartialEq"])],
    PARENT_DECLARES => [Expect::krate(&["crate", "Clone"])],
    PRIVATE_IMPORT => [Expect::std_trait(&["std", "hash", "Hash"]), Expect::unknown(&["Hash"])],
    REEXPORTED => [Expect::std_trait(&["std", "fmt", "Debug"]), Expect::unknown(&["crate", "b", "Hidden"])],
    CYCLE => [Expect::unknown(&["X"])],
    RE_EXPORT_CYCLE => [Expect::krate(&["crate", "Mark"]), Expect::std_trait(&["std", "clone", "Clone"])],
    WORKSPACE => [Expect::std_trait(&["std", "clone", "Clone"]), Expect::std_trait(&["std", "clone", "Clone"])],
    NESTED_FILES => [Expect::krate(&["crate", "Mark"]), Expect::krate(&["crate", "Mark"])],
]);

laws::table!(a_path_through_an_unplanted_module_is_unknown, Read::across, [
    ISLAND => [Expect::unknown(&["crate", "T"]), Expect::unknown(&["super", "T"])],
    ISLAND_OWN => [Expect::unknown(&["T"]), Expect::unknown(&["self", "T"]), Expect::unknown(&["m", "U"])],
    ISLAND_BESIDE_ROOT => [Expect::krate(&["crate", "T"]), Expect::unknown(&["T"])],
    PATHED => [Expect::unknown(&["x", "T"])],
    TWO_CANDIDATES => [Expect::unknown(&["x", "T"])],
]);

#[test]
fn a_test_module_file_is_planted_under_its_parent() {
    let tests = Read::file("src/tests.rs", "use super::*;\nimpl Default for R {}\n");
    let root = Read::file("src/lib.rs", "struct R;\n#[cfg(test)]\nmod tests;\n").with_test_modules(vec![tests]);
    assert_eq!(
        Read::implementations(&[root]),
        [Expect::written("R", Expect::std_trait(&["std", "default", "Default"]))]
    );
}

const FORMS: &str = "fn free() {}\nimpl S {\n    fn inherent(&self) -> bool { true }\n}\ntrait T {\n    fn declared(&self) -> u8;\n}\nimpl T for S {\n    fn declared(&self) -> u8 { 0 }\n}\nfn outer() {\n    fn nested() -> char { 'a' }\n    impl S {\n        fn inner(&self) {}\n    }\n}\n#[test]\nfn checked() {}\n#[tokio::test]\nasync fn awaited() {}\nfn main() {}\nmod m {\n    fn main() {}\n}\n";

const OWN_TYPE: &str = "struct Failure;\nfn f() -> Failure { Failure }\n";
const GENERIC_SHADOWS: &str = "struct T;\nfn f<T>() -> T { todo!() }\nfn g() -> T { T }\n";
const SELF_TYPE: &str = "struct S;\nimpl S {\n    fn new() -> Self { S }\n}\n";
const SELF_GENERIC: &str = "struct S<T>(T);\nimpl<T> S<T> {\n    fn get() -> Self { todo!() }\n}\n";
const SELF_TRAIT_IMPL: &str = "struct S;\nimpl Default for S {\n    fn default() -> Self { S }\n}\n";
const SELF_TRAIT_DECL: &str = "trait T {\n    fn f() -> Self;\n}\n";
const SELF_TRAIT_IN_IMPL: &str =
    "struct S;\nimpl S {\n    fn f() {\n        trait T {\n            fn g() -> Self;\n        }\n    }\n}\n";
const SELF_PROJECTED: &str = "struct S;\nimpl S {\n    fn f() -> Self::Out { todo!() }\n}\n";
const IMPL_GENERIC: &str = "struct S<T>(T);\nimpl<T> S<T> {\n    fn get(&self) -> Option<T> { None }\n}\n";
const PROJECTED: &str = "trait T {\n    type Out;\n    fn f(&self) -> Self::Out;\n}\n";

const CRATE_ALIAS: &str = "type Checked<T> = Result<T, std::fmt::Error>;\nfn f() -> Checked<u8> { todo!() }\n";
const STD_FMT_ALIAS: &str = "fn f() -> std::fmt::Result { Ok(()) }\n";
const STD_IO_ALIAS: &str = "use std::io;\nfn f() -> io::Result<()> { Ok(()) }\n";
const RECURSIVE_ALIAS: &str = "type A = B;\ntype B = A;\nfn f() -> A { todo!() }\n";
const DEFAULTED_ALIAS: &str =
    "struct Rejected;\ntype Res<T, E = Rejected> = Result<T, E>;\nfn f() -> Res<u8> { todo!() }\n";
const GIVEN_ALIAS: &str =
    "struct Rejected;\ntype Res<T, E = Rejected> = Result<T, E>;\nfn f() -> Res<u8, bool> { todo!() }\n";
const UNDEFAULTED_ALIAS: &str = "type Res<T, E> = Result<T, E>;\nfn f() -> Res<u8, bool> { todo!() }\n";
const SELF_DEFAULTED_ALIAS: &str =
    "struct Failure;\ntype Res<T, E = T> = Result<T, E>;\nfn f() -> Res<Failure> { todo!() }\n";
const WHOLE_DEFAULTED_ALIAS: &str =
    "struct Failure;\ntype Maybe<T, O = Option<T>> = O;\nfn f() -> Maybe<Failure> { todo!() }\n";
const DEFAULT_CHAINED_ALIAS: &str =
    "struct Failure;\ntype Tri<T, U = Option<T>, V = U> = V;\nfn f() -> Tri<Failure> { todo!() }\n";
const CHAINED_ALIAS: &str = "struct Failure;\ntype Outcome = Checked<()>;\ntype Checked<T> = Result<T, Failure>;\nfn f() -> Outcome { Ok(()) }\n";

const SUBJECTS: &str = "#[derive(Debug)]\nstruct A;\nimpl<T> Clone for Wrap<T> {}\n";

#[test]
fn every_function_is_read_with_its_owner_entry_and_output() {
    assert_eq!(
        Read::functions(FORMS),
        [
            Expect::function("free", Owner::Free, Entry::Called, None),
            Expect::function("inherent", Owner::Inherent, Entry::Called, Some(Expect::primitive(Primitive::Bool))),
            Expect::function("declared", Owner::TraitDecl, Entry::Called, Some(Expect::primitive(Primitive::Integer))),
            Expect::function("declared", Owner::TraitImpl, Entry::Called, Some(Expect::primitive(Primitive::Integer))),
            Expect::function("outer", Owner::Free, Entry::Called, None),
            Expect::function("nested", Owner::Nested, Entry::Called, Some(Expect::primitive(Primitive::Char))),
            Expect::function("inner", Owner::Inherent, Entry::Called, None),
            Expect::function("checked", Owner::Free, Entry::Test, None),
            Expect::function("awaited", Owner::Free, Entry::Test, None),
            Expect::function("main", Owner::Free, Entry::Called, None),
            Expect::function("main", Owner::Free, Entry::Called, None),
        ]
    );
}

laws::table!(a_written_output_resolves_in_the_scope_of_its_function, |text: &&str| Read::outputs(text), [
    OWN_TYPE => [Expect::named(Expect::item("Failure", ItemKind::Struct), Vec::new())],
    GENERIC_SHADOWS => [Expect::param("T"), Expect::named(Expect::item("T", ItemKind::Struct), Vec::new())],
    SELF_TYPE => [Expect::named(Expect::item("S", ItemKind::Struct), Vec::new())],
    SELF_GENERIC => [Expect::named(Expect::item("S", ItemKind::Struct), vec![Expect::param("T")])],
    SELF_TRAIT_IMPL => [Expect::named(Expect::item("S", ItemKind::Struct), Vec::new())],
    SELF_TRAIT_DECL => [Expect::param("Self")],
    SELF_TRAIT_IN_IMPL => [Expect::param("Self")],
    SELF_PROJECTED => [Expect::projected(Expect::named(Expect::item("S", ItemKind::Struct), Vec::new()))],
    IMPL_GENERIC => [Expect::named(Expect::std(&["std", "option", "Option"]), vec![Expect::param("T")])],
    PROJECTED => [Expect::projected(Expect::param("Self"))],
]);

const PAREN_PRIMITIVE: &str = "fn f() -> (bool) { true }\n";
const PLAIN_PRIMITIVE: &str = "fn f() -> bool { true }\n";
const PAREN_PATH: &str = "struct S;\nfn f() -> (S) { S }\n";
const PLAIN_PATH: &str = "struct S;\nfn f() -> S { S }\n";
const PAREN_GENERIC: &str = "fn f<T>() -> (Option<T>) { None }\n";
const PLAIN_GENERIC: &str = "fn f<T>() -> Option<T> { None }\n";
const PAREN_REFERENCE: &str = "fn f() -> (&'static str) { \"a\" }\n";
const PLAIN_REFERENCE: &str = "fn f() -> &'static str { \"a\" }\n";

laws::table!(a_parenthesised_output_reads_as_the_type_inside, |text: &&str| Read::outputs(text), [
    PAREN_PRIMITIVE => Read::outputs(PLAIN_PRIMITIVE),
    PAREN_PATH => Read::outputs(PLAIN_PATH),
    PAREN_GENERIC => Read::outputs(PLAIN_GENERIC),
    PAREN_REFERENCE => Read::outputs(PLAIN_REFERENCE),
]);

laws::table!(an_alias_expands_to_its_target_with_its_arguments, |text: &&str| Read::expanded(text), [
    CRATE_ALIAS => [Expect::named(
        Expect::std(&["std", "result", "Result"]),
        vec![Expect::primitive(Primitive::Integer), Expect::named(Expect::std(&["std", "fmt", "Error"]), Vec::new())],
    )],
    STD_FMT_ALIAS => [Expect::named(
        Expect::std(&["std", "result", "Result"]),
        vec![TypeRef::Tuple(Vec::new()), Expect::named(Expect::std(&["std", "fmt", "Error"]), Vec::new())],
    )],
    STD_IO_ALIAS => [Expect::named(
        Expect::std(&["std", "result", "Result"]),
        vec![TypeRef::Tuple(Vec::new()), Expect::named(Expect::std(&["std", "io", "Error"]), Vec::new())],
    )],
    RECURSIVE_ALIAS => [Expect::named(Expect::item("A", ItemKind::Alias), Vec::new())],
    CHAINED_ALIAS => [Expect::named(
        Expect::std(&["std", "result", "Result"]),
        vec![TypeRef::Tuple(Vec::new()), Expect::named(Expect::item("Failure", ItemKind::Struct), Vec::new())],
    )],
    DEFAULTED_ALIAS => [Expect::named(
        Expect::std(&["std", "result", "Result"]),
        vec![Expect::primitive(Primitive::Integer), Expect::named(Expect::item("Rejected", ItemKind::Struct), Vec::new())],
    )],
    SELF_DEFAULTED_ALIAS => [Expect::named(
        Expect::std(&["std", "result", "Result"]),
        vec![
            Expect::named(Expect::item("Failure", ItemKind::Struct), Vec::new()),
            Expect::named(Expect::item("Failure", ItemKind::Struct), Vec::new()),
        ],
    )],
    WHOLE_DEFAULTED_ALIAS => [Expect::named(
        Expect::std(&["std", "option", "Option"]),
        vec![Expect::named(Expect::item("Failure", ItemKind::Struct), Vec::new())],
    )],
    DEFAULT_CHAINED_ALIAS => [Expect::named(
        Expect::std(&["std", "option", "Option"]),
        vec![Expect::named(Expect::item("Failure", ItemKind::Struct), Vec::new())],
    )],
    GIVEN_ALIAS => [Expect::named(
        Expect::std(&["std", "result", "Result"]),
        vec![Expect::primitive(Primitive::Integer), Expect::primitive(Primitive::Bool)],
    )],
    UNDEFAULTED_ALIAS => [Expect::named(
        Expect::std(&["std", "result", "Result"]),
        vec![Expect::primitive(Primitive::Integer), Expect::primitive(Primitive::Bool)],
    )],
]);

const BINARY_MAIN: [Fixture; 1] = [Fixture::new("src/main.rs", "fn main() {}\n")];
const BINARY_LISTED_MAIN: [Fixture; 1] = [Fixture::new("src/bin/tool.rs", "fn main() {}\n")];
const LIBRARY_MAIN: [Fixture; 1] = [Fixture::new("src/lib.rs", "pub fn main() {}\n")];
const ISLAND_MAIN: [Fixture; 2] = [Fixture::new("src/lib.rs", "\n"), Fixture::new("src/loose.rs", "fn main() {}\n")];
const BINARY_CHILD_MAIN: [Fixture; 2] =
    [Fixture::new("src/main.rs", "mod m;\n"), Fixture::new("src/m.rs", "fn main() {}\n")];

laws::table!(only_the_main_of_a_binary_crate_root_is_the_entry, Read::entries, [
    BINARY_MAIN => [Expect::function("main", Owner::Free, Entry::Main, None)],
    BINARY_LISTED_MAIN => [Expect::function("main", Owner::Free, Entry::Main, None)],
    LIBRARY_MAIN => [Expect::function("main", Owner::Free, Entry::Called, None)],
    ISLAND_MAIN => [Expect::function("main", Owner::Free, Entry::Called, None)],
    BINARY_CHILD_MAIN => [Expect::function("main", Owner::Free, Entry::Called, None)],
]);

#[test]
fn an_implementation_reads_the_type_it_implements_for() {
    assert_eq!(
        Read::subjects(SUBJECTS),
        [
            Expect::named(Expect::item("A", ItemKind::Struct), Vec::new()),
            Expect::named(Expect::unknown(&["Wrap"]), vec![Expect::param("T")]),
        ]
    );
}
