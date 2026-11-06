use code_model::{
    Access, Arrayed, Bounds, Callable, Callee, Held, ItemPath, MacroCall, Named, Owner, PathCall, Position, Primitive,
    Projection, Reach, Referenced, Region, Resolved, Seat, Sited, Spelling, TypeRef, Within, Zone,
};
use code_reader::Reader;
use lint_core::{SourceFile, SourceText};
use lint_layout::{LineNumber, Name};

struct Read;

impl Read {
    fn model(text: &str) -> code_model::Model {
        match SourceFile::parse("a.rs", SourceText::new(text.to_owned())) {
            Ok(file) => Reader::read_files(&[file]),
            Err(failure) => unreachable!("the source parses: {failure}"),
        }
    }

    fn written(text: &str) -> Vec<TypeRef<Resolved>> {
        let model = Read::model(text);
        model
            .functions()
            .flat_map(|found| found.entity().signature().written().cloned().collect::<Vec<TypeRef>>())
            .map(Read::resolved)
            .collect()
    }

    fn resolved(ty: TypeRef) -> TypeRef<Resolved> {
        ty.mapped(&mut |sited: Sited| AsRef::<Resolved>::as_ref(&sited).clone())
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
                slotted: found.entity().slot().map(|_| Declares::Item),
            })
            .collect()
    }

    fn placed(text: &str) -> Vec<PlacedRow> {
        let model = Read::model(text);
        model
            .functions()
            .everywhere()
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
    slotted: Option<Declares>,
}

impl HeldRow {
    fn new(name: &str, held: Held) -> HeldRow {
        HeldRow { name: name.to_owned(), held, declares: None, slotted: None }
    }

    fn declaring(name: &str, held: Held) -> HeldRow {
        HeldRow { name: name.to_owned(), held, declares: Some(Declares::Item), slotted: None }
    }

    fn slotted(name: &str, held: Held) -> HeldRow {
        HeldRow { name: name.to_owned(), held, declares: None, slotted: Some(Declares::Item) }
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

    fn std(segments: &[&str]) -> Resolved {
        Resolved::Std(Expect::path(segments))
    }

    fn referenced(access: Access, pointee: TypeRef<Resolved>) -> TypeRef<Resolved> {
        TypeRef::Reference(Referenced::new(access, pointee))
    }

    fn sited(segments: &[&str], line: usize) -> Spelling {
        Spelling::Written(Expect::path(segments), LineNumber::from(line))
    }

    fn call(site: &str, callee: Callee, reach: Reach) -> CalledRow {
        CalledRow { site: site.to_owned(), callee, reach }
    }
}

const REFERENCE: &str = "fn f(x: &u8) {}\n";
const EXCLUSIVE: &str = "fn f(x: &mut u8) {}\n";
const POINTER: &str = "fn f(x: *const u8) {}\n";
const SLICE: &str = "fn f(x: &[u8]) {}\n";
const ARRAY: &str = "fn f(x: [u8; 4]) {}\n";
const MEASURED: &str = "fn f(x: [u8; N as usize]) {}\n";
const IMPLEMENTING: &str = "fn f(x: impl Into<u8>) {}\n";
const DYNAMIC: &str = "fn f(x: &dyn Fn(char) -> bool) {}\n";
const BOUND_ITEM: &str = "fn f(x: impl Iterator<Item = u8>) {}\n";
const QUALIFIED: &str = "fn f(x: <S as T>::X) {}\n";
const BARE_FUNCTION: &str = "fn f(x: fn(u8) -> bool) {}\n";
const MACRO_TYPE: &str = "fn f(x: m!()) {}\n";

laws::table!(a_type_is_lowered_into_its_structured_form, |text: &&str| Read::written(text), [
    REFERENCE => [Expect::referenced(Access::Shared, Expect::integer())],
    EXCLUSIVE => [Expect::referenced(Access::Exclusive, Expect::integer())],
    POINTER => [TypeRef::Pointer(Referenced::new(Access::Shared, Expect::integer()))],
    SLICE => [Expect::referenced(Access::Shared, TypeRef::Slice(Box::new(Expect::integer())))],
    ARRAY => [TypeRef::Array(Arrayed::new(Expect::integer(), Vec::new()))],
    MEASURED => [TypeRef::Array(Arrayed::new(Expect::integer(), vec![Expect::integer()]))],
    IMPLEMENTING => [TypeRef::Bounded(Bounds::from(vec![Named::new(
        Expect::std(&["std", "convert", "Into"]),
        vec![Expect::integer()],
    )]))],
    DYNAMIC => [Expect::referenced(Access::Shared, TypeRef::Bounded(Bounds::from(vec![Named::bound(
        Expect::std(&["std", "ops", "Fn"]),
        vec![Expect::primitive(Primitive::Char)],
        vec![Expect::primitive(Primitive::Bool)],
    )])))],
    BOUND_ITEM => [TypeRef::Bounded(Bounds::from(vec![Named::bound(
        Expect::std(&["std", "iter", "Iterator"]),
        Vec::new(),
        vec![Expect::integer()],
    )]))],
    QUALIFIED => [TypeRef::Projected(Box::new(Projection::new(
        Expect::named(Resolved::Unknown(Expect::path(&["S"])), Vec::new()),
        Some(Named::new(Resolved::Unknown(Expect::path(&["T"])), Vec::new())),
        Name::from("X"),
        LineNumber::from(1),
    )))],
    BARE_FUNCTION => [TypeRef::Function(Callable::new(vec![Expect::integer()], Expect::primitive(Primitive::Bool)))],
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
            TypeRef::Bounded(Bounds::from(vec![Named::new(
                Expect::std(&["std", "convert", "Into"]),
                vec![Expect::integer()]
            )])),
            Expect::integer(),
            Expect::primitive(Primitive::Bool),
            parameter("T"),
            TypeRef::Bounded(Bounds::from(vec![Named::new(Expect::std(&["std", "marker", "Copy"]), Vec::new())])),
            parameter("T"),
            parameter("U"),
        ]
    );
}

const SPELLED: &str = "fn f(x: core::primitive::u8, y: u8) {}\n";
const SELF_SPELLED: &str = "struct S;\nimpl S {\n    fn f() -> Self {\n        S\n    }\n}\n";

#[test]
fn a_qualified_primitive_resolves_like_the_bare_one_and_keeps_its_spelling() {
    let integer = Resolved::Primitive(Primitive::Integer);
    assert_eq!(
        Read::spelt(SPELLED),
        [
            Sited::new(integer.clone(), Expect::sited(&["core", "primitive", "u8"], 1)),
            Sited::new(integer, Expect::sited(&["u8"], 1)),
        ]
    );
}

#[test]
fn self_keeps_its_spelling_and_line() {
    let spelt = Read::spelt(SELF_SPELLED);
    let spellings: Vec<&Spelling> = spelt.iter().map(AsRef::<Spelling>::as_ref).collect();
    assert_eq!(spellings, [&Spelling::SelfType(LineNumber::from(3))]);
}

const HOLDINGS: &str = "struct Pair<T = u8>(T, u8);\nstruct Count(usize);\nenum E {\n    A(u8),\n    B { b: u8 },\n}\nunion U {\n    a: u8,\n}\nconst C: u8 = 1;\nstatic S: u8 = 1;\nunsafe extern \"C\" {\n    static F: u8;\n}\ntype A = u8;\ntrait T {\n    type X: Into<u8>;\n    const N: u8;\n}\nimpl T for Count {\n    type X = u8;\n    const N: u8 = 1;\n}\n";

#[test]
fn every_item_that_holds_a_type_is_read_with_its_kind() {
    assert_eq!(
        Read::held(HOLDINGS),
        [
            HeldRow::declaring("Pair", Held::StructParameter),
            HeldRow::new("Pair", Held::StructField),
            HeldRow::new("Pair", Held::StructField),
            HeldRow::declaring("Count", Held::NewtypeField),
            HeldRow::new("A", Held::EnumField),
            HeldRow::new("b", Held::EnumField),
            HeldRow::new("a", Held::UnionField),
            HeldRow::new("C", Held::Const),
            HeldRow::new("S", Held::Static),
            HeldRow::new("F", Held::Static),
            HeldRow::declaring("A", Held::Alias),
            HeldRow::new("X", Held::AssociatedType),
            HeldRow::new("N", Held::AssociatedConst),
            HeldRow::slotted("X", Held::AssociatedType),
            HeldRow::slotted("N", Held::AssociatedConst),
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
            PlacedRow::new("expanded", Owner::Free, Reach::MacroBody),
            PlacedRow::new("parsed", Owner::Free, Reach::Shipped).zoned(Zone::Boundary),
            PlacedRow::new("foreign", Owner::Foreign, Reach::Shipped),
            PlacedRow::new("clone", Owner::TraitImpl, Reach::Shipped).inside(Within::TraitImpl),
            PlacedRow::new("helper", Owner::Nested, Reach::Shipped).inside(Within::TraitImpl),
        ]
    );
}

const CALLED: &str = "fn f(x: Name) {\n    x.trim();\n    String::from(\"a\");\n    format!(\"{}\", x.len());\n    let y = format!(\"b\");\n}\n";

#[test]
fn every_call_is_read_with_std_macro_arguments_at_macro_body_reach() {
    let owner = TypeRef::Named(Named::new(
        Sited::new(Expect::std(&["std", "string", "String"]), Expect::sited(&["String"], 3)),
        Vec::new(),
    ));
    assert_eq!(
        Read::called(CALLED),
        [
            Expect::call("a.rs:2", Callee::Method(Name::from("trim")), Reach::Shipped),
            Expect::call(
                "a.rs:3",
                Callee::Path(PathCall::new(Expect::path(&["String", "from"]), owner)),
                Reach::Shipped
            ),
            Expect::call(
                "a.rs:4",
                Callee::Macro(MacroCall::new(Expect::path(&["format"]), Position::Statement)),
                Reach::Shipped
            ),
            Expect::call("a.rs:4", Callee::Method(Name::from("len")), Reach::MacroBody),
            Expect::call(
                "a.rs:5",
                Callee::Macro(MacroCall::new(Expect::path(&["format"]), Position::Expression)),
                Reach::Shipped
            ),
        ]
    );
}
