use std::collections::BTreeSet;

use graph::{
    Arrayed, Callee, Exposure, Held, Item, ItemKind, ItemPath, MacroCall, Mention, MethodShape, Named, Owner,
    ParamIndex, Primitive, Reach, Receiver, Region, Resolved, Returned, Seat, Sited, Spelling, Taken, TypeRef,
    Unplaced, Within, Zone,
};
use graph_reader::Reader;
use lint_core::{SourceFile, SourceText};
use lint_layout::{Failure, Name};

struct Read;

impl Read {
    fn model(text: &str) -> graph::Model {
        match SourceFile::parse("a.rs", SourceText::new(text.to_owned())) {
            Ok(file) => Reader::read_files(&[file]),
            Err(failure) => unreachable!("the source parses: {failure}"),
        }
    }

    fn supported(root: &str, module: &str) -> Result<SourceFile, Failure> {
        let parsed = SourceFile::parse("src/support.rs", SourceText::new(module.to_owned()));
        parsed.and_then(|support| {
            SourceFile::parse("src/lib.rs", SourceText::new(root.to_owned()))
                .map(|lib| lib.with_test_modules(vec![support]))
        })
    }

    fn reaches(root: &str, module: &str) -> Vec<Reach> {
        match Read::supported(root, module) {
            Ok(file) => Reader::read_files(&[file])
                .functions()
                .map(|found| AsRef::<Region>::as_ref(found.entity()).reach())
                .collect(),
            Err(failure) => unreachable!("the sources parse: {failure}"),
        }
    }

    fn written(text: &str) -> Vec<TypeRef<Resolved>> {
        let model = Read::model(text);
        model
            .functions()
            .flat_map(|found| found.entity().signature().written().map(Read::resolved).collect::<Vec<_>>())
            .collect()
    }

    fn resolved(ty: &TypeRef) -> TypeRef<Resolved> {
        ty.mapped(|sited: &Sited| AsRef::<Resolved>::as_ref(sited).clone())
    }

    fn spelt(text: &str) -> Vec<Sited> {
        let model = Read::model(text);
        model
            .functions()
            .flat_map(|found| found.entity().signature().written().cloned().collect::<Vec<TypeRef>>())
            .filter_map(|ty| ty.named().map(|named| AsRef::<Sited>::as_ref(named).clone()))
            .collect()
    }

    fn held(text: &str) -> Vec<HeldRow> {
        let model = Read::model(text);
        model
            .holdings()
            .map(|found| HeldRow {
                name: AsRef::<Name>::as_ref(found.entity()).to_string(),
                held: found.entity().held(),
                declares: found.entity().declares().and_then(|item| model.declared(item)).map(|_| Declares::Item),
            })
            .collect()
    }

    fn placed(text: &str) -> Vec<PlacedRow> {
        let model = Read::model(text);
        model
            .functions()
            .map(|found| {
                let region: &Region = found.entity().as_ref();
                PlacedRow {
                    name: AsRef::<Name>::as_ref(found.entity()).to_string(),
                    owner: found.entity().owner(),
                    reach: region.reach(),
                    zone: region.zone(),
                    within: region.within(),
                }
            })
            .collect()
    }

    fn called(text: &str) -> Vec<CalledRow> {
        let model = Read::model(text);
        model
            .calls()
            .map(|found| {
                let seat: &Seat = found.entity().as_ref();
                let region: &Region = seat.as_ref();
                CalledRow {
                    site: found.site().to_string(),
                    callee: found.entity().callee().clone(),
                    reach: region.reach(),
                }
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Declares {
    Item,
}

#[derive(Debug, PartialEq)]
struct HeldRow {
    name: String,
    held: Held,
    declares: Option<Declares>,
}

impl HeldRow {
    fn declaring(name: &str, held: Held) -> HeldRow {
        HeldRow { name: name.to_owned(), held, declares: Some(Declares::Item) }
    }
}

#[derive(Debug, PartialEq)]
struct PlacedRow {
    name: String,
    owner: Owner,
    reach: Reach,
    zone: Zone,
    within: Within,
}

impl PlacedRow {
    fn new(name: &str, owner: Owner, reach: Reach) -> PlacedRow {
        PlacedRow { name: name.to_owned(), owner, reach, zone: Zone::Open, within: Within::Own }
    }

    fn zoned(self, zone: Zone) -> PlacedRow {
        PlacedRow { name: self.name, owner: self.owner, reach: self.reach, zone, within: self.within }
    }

    fn inside(self, within: Within) -> PlacedRow {
        PlacedRow { name: self.name, owner: self.owner, reach: self.reach, zone: self.zone, within }
    }
}

#[derive(Debug, PartialEq)]
struct CalledRow {
    site: String,
    callee: Callee,
    reach: Reach,
}

struct Expect;

impl Expect {
    fn path(segments: &[&str]) -> ItemPath {
        segments.iter().map(|segment| Name::from(*segment)).collect()
    }

    fn named(resolved: Resolved, arguments: Vec<TypeRef<Resolved>>) -> TypeRef<Resolved> {
        TypeRef::Named(Named::new(resolved, arguments))
    }

    fn integer() -> TypeRef<Resolved> {
        Expect::named(Resolved::Primitive(Primitive::Integer), Vec::new())
    }

    fn primitive(primitive: Primitive) -> TypeRef<Resolved> {
        Expect::named(Resolved::Primitive(primitive), Vec::new())
    }

    fn std_trait(segments: &[&str]) -> Resolved {
        Resolved::StdTrait(Expect::path(segments))
    }

    fn referenced(pointee: TypeRef<Resolved>) -> TypeRef<Resolved> {
        TypeRef::Reference(Box::new(pointee))
    }

    fn call(site: &str, callee: Callee, reach: Reach) -> CalledRow {
        CalledRow { site: site.to_owned(), callee, reach }
    }
}

const REFERENCE: &str = "fn f(x: &u8) {}\n";
const POINTER: &str = "fn f(x: *const u8) {}\n";
const SLICE: &str = "fn f(x: &[u8]) {}\n";
const ARRAY: &str = "fn f(x: [u8; 4]) {}\n";
const MEASURED: &str = "fn f(x: [u8; N as usize]) {}\n";
const IMPLEMENTING: &str = "fn f(x: impl Into<u8>) {}\n";
const DYNAMIC: &str = "fn f(x: &dyn Fn(char) -> bool) {}\n";
const BOUND_ITEM: &str = "fn f(x: impl Iterator<Item = u8>) {}\n";
const MACRO_TYPE: &str = "fn f(x: m!()) {}\n";

laws::table!(a_type_is_lowered_into_its_structured_form, |text: &&str| Read::written(text), [
    REFERENCE => [Expect::referenced(Expect::integer())],
    POINTER => [TypeRef::Pointer(Box::new(Expect::integer()))],
    SLICE => [Expect::referenced(TypeRef::Slice(Box::new(Expect::integer())))],
    ARRAY => [TypeRef::Array(Arrayed::new(Expect::integer(), Vec::new()))],
    MEASURED => [TypeRef::Array(Arrayed::new(Expect::integer(), vec![Expect::integer()]))],
    IMPLEMENTING => [TypeRef::Bounded(vec![Expect::named(
        Expect::std_trait(&["std", "convert", "Into"]),
        vec![Expect::integer()],
    )])],
    DYNAMIC => [Expect::referenced(TypeRef::Bounded(vec![TypeRef::Named(Named::bound(
        Expect::std_trait(&["std", "ops", "Fn"]),
        vec![Expect::primitive(Primitive::Char)],
        vec![Expect::primitive(Primitive::Bool)],
    ))]))],
    BOUND_ITEM => [TypeRef::Bounded(vec![TypeRef::Named(Named::bound(
        Expect::std_trait(&["std", "iter", "Iterator"]),
        Vec::new(),
        vec![Expect::integer()],
    ))])],
    MACRO_TYPE => [TypeRef::Opaque],
]);

const CONSTRAINED: &str =
    "fn f<T: Into<u8>, const N: usize, U = bool>(x: T) -> U\nwhere\n    T: Copy,\n{\n    g(x)\n}\n";

#[test]
fn a_signature_reads_its_generic_constraints_inputs_and_output() {
    let parameter = |name: &str| Expect::named(Resolved::Param(Name::from(name)), Vec::new());
    assert_eq!(
        Read::written(CONSTRAINED),
        [
            TypeRef::Bounded(vec![Expect::named(
                Expect::std_trait(&["std", "convert", "Into"]),
                vec![Expect::integer()]
            )]),
            Expect::integer(),
            Expect::primitive(Primitive::Bool),
            parameter("T"),
            TypeRef::Bounded(vec![Expect::named(Expect::std_trait(&["std", "marker", "Copy"]), Vec::new())]),
            parameter("T"),
            parameter("U"),
        ]
    );
}

const NESTED_TRAIT: &str = "fn f<T>() {\n    trait Q {\n        fn g(&self, x: T);\n    }\n}\n";

#[test]
fn a_trait_nested_in_a_generic_function_keeps_the_function_type_parameters() {
    let parameter = Expect::named(Resolved::Param(Name::from("T")), Vec::new());
    assert!(Read::written(NESTED_TRAIT).contains(&parameter));
}

const SELF_SPELLED: &str = "struct S;\nimpl S {\n    fn f() -> Self {\n        S\n    }\n}\n";

#[test]
fn self_is_not_a_written_spelling() {
    let spelt = Read::spelt(SELF_SPELLED);
    let spellings: Vec<&Spelling> = spelt.iter().map(AsRef::<Spelling>::as_ref).collect();
    assert_eq!(spellings, [&Spelling::Implied]);
}

const HOLDINGS: &str = "struct Pair<T = u8>(T, u8);\nstruct Count(usize);\nenum E<T = u8> {\n    A(u8),\n    B { b: u8 },\n}\nunion U {\n    a: u8,\n}\nconst C: u8 = 1;\ntype A = u8;\n";

#[test]
fn every_field_bound_constant_and_alias_is_read_with_its_kind_and_item() {
    assert_eq!(
        Read::held(HOLDINGS),
        [
            HeldRow::declaring("Pair", Held::StructParameter),
            HeldRow::declaring("Pair", Held::StructField),
            HeldRow::declaring("Pair", Held::StructField),
            HeldRow::declaring("Count", Held::NewtypeField),
            HeldRow::declaring("E", Held::Bound),
            HeldRow::declaring("A", Held::Variant),
            HeldRow::declaring("A", Held::EnumField),
            HeldRow::declaring("B", Held::Variant),
            HeldRow::declaring("b", Held::EnumField),
            HeldRow::declaring("a", Held::UnionField),
            HeldRow::declaring("C", Held::Value),
            HeldRow::declaring("A", Held::Alias),
        ]
    );
}

const PLACED: &str = "fn shipped() {}\n#[cfg(test)]\nmod tests {\n    fn tested() {}\n}\nm! {\n    fn expanded() {}\n}\nmod boundary {\n    fn parsed() {}\n}\nunsafe extern \"C\" {\n    fn foreign(a: u8);\n}\nimpl Clone for S {\n    fn clone(&self) -> S {\n        fn helper() {}\n        S\n    }\n}\n";

#[test]
fn every_function_carries_its_reach_zone_and_enclosure() {
    assert_eq!(
        Read::placed(PLACED),
        [
            PlacedRow::new("shipped", Owner::Free, Reach::Shipped),
            PlacedRow::new("tested", Owner::Free, Reach::Test),
            PlacedRow::new("expanded", Owner::Free, Reach::Shipped),
            PlacedRow::new("parsed", Owner::Free, Reach::Shipped).zoned(Zone::Boundary),
            PlacedRow::new("foreign", Owner::Foreign, Reach::Shipped),
            PlacedRow::new("clone", Owner::TraitImpl, Reach::Shipped).inside(Within::TraitImpl),
            PlacedRow::new("helper", Owner::Nested, Reach::Shipped).inside(Within::TraitImpl),
        ]
    );
}

const CALLED: &str = "fn f(x: Name) {\n    x.trim();\n    String::from(\"a\");\n    format!(\"{}\", x.len());\n    let y = format!(\"b\");\n}\n";

#[test]
fn every_method_path_and_expression_macro_call_is_read() {
    assert_eq!(
        Read::called(CALLED),
        [
            Expect::call("a.rs:2", Callee::Method(Name::from("trim")), Reach::Shipped),
            Expect::call("a.rs:3", Callee::Path(Expect::path(&["String", "from"])), Reach::Shipped),
            Expect::call("a.rs:5", Callee::Macro(MacroCall::from(Expect::path(&["format"]))), Reach::Shipped),
        ]
    );
}

const SUPPORTED: &str = "pub fn root() {}\n#[cfg(test)]\nmod support;\n";
const SUPPORT: &str = "pub fn helper() {}\n";

#[test]
fn a_test_module_file_is_read_at_test_reach() {
    assert_eq!(Read::reaches(SUPPORTED, SUPPORT), [Reach::Shipped, Reach::Test]);
}

const GATED_ROOT: &str = "pub fn root() {}\n#[cfg(test)]\nmod gated;\n";
const GATED: &str = "pub fn middle() {}\nmod deeper;\n";
const DEEPER: &str = "pub fn deep() {}\n";

#[test]
fn a_module_file_below_a_test_gated_module_file_is_read_at_test_reach() {
    let files = [("src/lib.rs", GATED_ROOT), ("src/gated.rs", GATED), ("src/gated/deeper.rs", DEEPER)]
        .map(|(path, text)| SourceFile::parse(path, SourceText::new(text.to_owned())));
    let parsed: Vec<SourceFile> = files.into_iter().filter_map(Result::ok).collect();
    let reaches: BTreeSet<(String, Reach)> = Reader::read_files(&parsed)
        .functions()
        .map(|found| (found.site().to_string(), AsRef::<Region>::as_ref(found.entity()).reach()))
        .collect();
    assert_eq!(
        reaches,
        BTreeSet::from([
            ("src/gated.rs:1".to_owned(), Reach::Test),
            ("src/gated/deeper.rs:1".to_owned(), Reach::Test),
            ("src/lib.rs:1".to_owned(), Reach::Shipped),
        ])
    );
}

#[test]
fn a_macro_in_statement_position_quotes_its_first_literal_like_one_in_an_expression() {
    let model = Read::model(
        "fn f() {\n    other::query!(Row, \"SELECT 1\", id: Id);\n    g(other::query!(Row, \"SELECT 2\"));\n}\n",
    );
    let quoted: Vec<String> =
        model.quotations().map(|placed| AsRef::<graph::Quote>::as_ref(placed.entity()).as_ref().to_owned()).collect();
    assert_eq!(quoted, vec!["\"SELECT 1\"".to_owned(), "\"SELECT 2\"".to_owned()]);
}

const IN_MACRO: &str = "fn field() {}\nfn c() {}\nfn f(x: S) {\n    g!(x.field, a::b, -c);\n}\n";

#[test]
fn a_name_after_a_dot_in_macro_tokens_is_a_field_or_method_not_a_path() {
    let model = Read::model(IN_MACRO);
    let mut written: Vec<String> = model
        .uses()
        .filter(|placed| placed.entity().mention() == Mention::InMacro)
        .map(|placed| placed.entity().written().to_string())
        .collect();
    written.sort();
    assert_eq!(written, ["a::b", "c"]);
}

#[test]
fn a_member_named_through_a_type_alias_belongs_to_the_aliased_type() {
    let model = Read::model(
        "pub struct InFile;\nimpl InFile {\n    pub fn new() -> InFile {\n        InFile\n    }\n}\npub type Site = InFile;\nfn f() {\n    Site::new();\n}\n",
    );
    let owners: Vec<String> = model
        .uses()
        .filter_map(|placed| placed.entity().resolved().member().map(graph::Member::owner))
        .filter_map(|owner| model.declared(owner))
        .map(|item| AsRef::<lint_layout::Name>::as_ref(item).to_string())
        .collect();
    assert_eq!(owners, ["InFile"]);
}

#[test]
fn a_shared_adaptor_chained_on_another_still_repeats_per_element() {
    let model = Read::model(
        "fn f(step: u8, ids: Vec<u8>) -> Vec<u8> {\n    ids.iter().filter(|id| **id > 0).map(|id| step.wrapping_add(*id)).collect()\n}\n",
    );
    let mut methods: Vec<String> = model
        .functions()
        .filter_map(|placed| placed.entity().definition())
        .flat_map(|definition| definition.repetitions().iter().filter_map(|repetition| repetition.method()))
        .map(ToString::to_string)
        .collect();
    methods.sort();
    assert_eq!(methods, ["wrapping_add"]);
}

#[test]
fn a_self_field_read_in_a_loop_repeats_per_pass() {
    let model = Read::model(
        "struct Counter {\n    limit: u8,\n}\nimpl Counter {\n    fn f(&self, ids: Vec<u8>) -> u8 {\n        ids.iter().map(|id| id.min(&self.limit)).sum()\n    }\n}\n",
    );
    let fields: Vec<String> = model
        .functions()
        .filter_map(|placed| placed.entity().definition())
        .flat_map(|definition| {
            definition.repetitions().iter().filter(|repetition| repetition.reach().index().is_none())
        })
        .map(|repetition| AsRef::<lint_layout::Name>::as_ref(repetition.reach()).to_string())
        .collect();
    assert_eq!(fields, ["limit"]);
}

#[test]
fn a_struct_literal_uses_its_type_path() {
    let model = Read::model("struct Point {\n    x: u8,\n}\nfn f() -> Point {\n    Point { x: 1 }\n}\n");
    let literals: Vec<String> = model
        .uses()
        .filter(|placed| placed.entity().mention() == Mention::Expression)
        .map(|placed| format!("{} {}", placed.entity().written(), placed.site()))
        .collect();
    assert_eq!(literals, ["Point a.rs:5"]);
}

const TRANSCRIBED: &str = "pub struct Kept;\nmod inner {\n    pub struct Thing;\n}\nuse inner::Thing;\nmacro_rules! made {\n    ($($name:ident),*) => {\n        $(fn $name() -> crate::Kept {\n            Thing::from(Gone)\n        })*\n    };\n}\nmade!(f, g);\n";

#[test]
fn an_invocation_names_what_its_macro_transcriber_names_where_it_resolves() {
    let model = Read::model(TRANSCRIBED);
    let mut expanded: Vec<String> = model
        .uses()
        .filter(|placed| placed.entity().mention() == Mention::Expanded)
        .map(|placed| placed.entity().written().to_string())
        .collect();
    expanded.sort();
    assert_eq!(expanded, ["Thing::from", "crate::Kept"]);
}

const DEFINING: &str = "pub struct Kept;\n#[macro_export]\nmacro_rules! made {\n    () => {\n        fn f() -> $crate::Kept {\n            todo!()\n        }\n    };\n}\n";
const INVOKING: &str = "pub enum Kept {}\na::made!();\n";

#[test]
fn a_dollar_crate_path_is_used_by_the_defining_crate_not_by_the_crate_that_invokes_the_macro() -> Result<(), Failure> {
    let files = [
        SourceFile::parse("crates/a/src/lib.rs", SourceText::new(DEFINING.to_owned()))?,
        SourceFile::parse("crates/b/src/lib.rs", SourceText::new(INVOKING.to_owned()))?,
    ];
    let model = Reader::read_files(&files);
    let expanded: Vec<(String, Option<Item>)> = model
        .uses()
        .filter(|placed| placed.entity().mention() == Mention::Expanded)
        .map(|placed| {
            let item = placed.entity().resolved().crate_item().and_then(|item| model.declared(item)).cloned();
            (placed.site().to_string(), item)
        })
        .collect();
    let kept =
        Unplaced::new(Name::from("Kept"), ItemKind::Struct).placed(Vec::new(), Some(Name::from("a")), Exposure::Public);
    assert_eq!(expanded, [("crates/a/src/lib.rs:5".to_owned(), Some(kept))]);
    Ok(())
}

#[test]
fn a_method_call_beside_the_tail_call_keeps_the_whole_parameter_read() {
    let model = Read::model(
        "struct X;\nstruct Y;\nfn helper(n: u8, x: &X) -> u8 {\n    n\n}\nfn f(x: &X, y: &Y) -> u8 {\n    helper(y.size(), x)\n}\n",
    );
    let reads: Vec<ParamIndex> = model
        .functions()
        .filter(|placed| AsRef::<Name>::as_ref(placed.entity()) == &Name::from("f"))
        .filter_map(|placed| placed.entity().definition())
        .flat_map(|definition| definition.reads().iter().map(graph::Read::parameter))
        .collect();
    assert_eq!(reads, [ParamIndex::from(0)]);
}

#[test]
fn a_free_function_body_resolves_the_function_type_parameters() {
    let model = Read::model("fn f<T: Clone>(x: &T) -> T {\n    x.clone()\n}\n");
    let receivers_are_the_parameter: Vec<bool> = model
        .functions()
        .filter_map(|placed| placed.entity().definition())
        .flat_map(|definition| definition.invocations().iter().map(|invocation| format!("{invocation:?}")))
        .map(|shown| shown.contains("resolved: Param(Name(\"T\"))"))
        .collect();
    assert_eq!(receivers_are_the_parameter, [true]);
}

#[test]
fn a_method_shape_counts_only_its_own_type_written_by_name_as_itself() {
    let model = Read::model(
        "struct A;\nstruct B;\nimpl A {\n    fn join(self, other: A, by: B) -> A {\n        other\n    }\n}\n",
    );
    let shapes: Vec<MethodShape> = model.functions().map(|placed| MethodShape::from(placed.entity())).collect();
    assert_eq!(shapes, [MethodShape::new(Some(Receiver::Owned), vec![Taken::Itself, Taken::Other], Returned::Itself)]);
}
