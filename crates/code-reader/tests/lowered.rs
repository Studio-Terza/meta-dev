use code_model::{
    Arrayed, Callee, Held, ItemPath, MacroCall, Named, Owner, Primitive, Reach, Region, Resolved, Seat, Sited,
    Spelling, TypeRef, Within, Zone,
};
use code_reader::Reader;
use lint_core::{SourceFile, SourceText};
use lint_layout::{Failure, Name};

struct Read;

impl Read {
    fn model(text: &str) -> code_model::Model {
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
        ty.mapped(&mut |sited: &Sited| AsRef::<Resolved>::as_ref(sited).clone())
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

const SELF_SPELLED: &str = "struct S;\nimpl S {\n    fn f() -> Self {\n        S\n    }\n}\n";

#[test]
fn self_is_not_a_written_spelling() {
    let spelt = Read::spelt(SELF_SPELLED);
    let spellings: Vec<&Spelling> = spelt.iter().map(AsRef::<Spelling>::as_ref).collect();
    assert_eq!(spellings, [&Spelling::Implied]);
}

const HOLDINGS: &str = "struct Pair<T = u8>(T, u8);\nstruct Count(usize);\nenum E<T = u8> {\n    A(u8),\n    B { b: u8 },\n}\nunion U {\n    a: u8,\n}\nconst C: u8 = 1;\ntype A = u8;\n";

#[test]
fn every_field_struct_parameter_and_alias_is_read_with_its_kind_and_item() {
    assert_eq!(
        Read::held(HOLDINGS),
        [
            HeldRow::declaring("Pair", Held::StructParameter),
            HeldRow::declaring("Pair", Held::StructField),
            HeldRow::declaring("Pair", Held::StructField),
            HeldRow::declaring("Count", Held::NewtypeField),
            HeldRow::declaring("A", Held::EnumField),
            HeldRow::declaring("b", Held::EnumField),
            HeldRow::declaring("a", Held::UnionField),
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
