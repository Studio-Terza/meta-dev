use syn::Macro;

use code_model::{Alias, AliasParameter, Building, ItemPath, Named, Primitive, Resolved, Sited, TypeRef};
use lint_layout::{Name, Whole, Word, words};

use super::binding::Binding;
use super::tables::{Key, Keyed};
use super::view::View;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Trait,
    Type,
}

impl Kind {
    fn resolved(found: Option<Self>, path: ItemPath) -> Resolved {
        match found {
            Some(Self::Trait) => Resolved::StdTrait(path),
            Some(Self::Type) | None => Resolved::Std(path),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Member {
    name: Word<'static>,
    kind: Kind,
}

impl Member {
    const fn new(name: Word<'static>, kind: Kind) -> Self {
        Self { name, kind }
    }

    fn kind(&self, name: &Name) -> Option<Kind> {
        name.matches::<Whole>(&[self.name]).map(|_| self.kind)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Canonical {
    module: Word<'static>,
    name: Word<'static>,
    kind: Kind,
}

impl Canonical {
    const fn new(module: Word<'static>, name: Word<'static>, kind: Kind) -> Self {
        Self { module, name, kind }
    }

    fn kind(&self, module: &Name, name: &Name) -> Option<Kind> {
        module.matches::<Whole>(&[self.module]).and(name.matches::<Whole>(&[self.name])).map(|_| self.kind)
    }

    pub(super) fn path(&self) -> ItemPath {
        [StdNames::STD, self.module, self.name].map(Name::from).into_iter().collect()
    }
}

#[derive(Clone, Copy, Debug)]
struct PrimitiveName {
    word: Word<'static>,
    primitive: Primitive,
}

impl PrimitiveName {
    const fn new(word: Word<'static>, primitive: Primitive) -> Self {
        Self { word, primitive }
    }
}

#[derive(Clone, Copy, Debug)]
enum OkSide {
    Unit,
    Parameter,
}

impl OkSide {
    fn parameters(self) -> Vec<AliasParameter> {
        match self {
            Self::Unit => Vec::new(),
            Self::Parameter => vec![AliasParameter::new(Name::from(StdAlias::PARAMETER), None)],
        }
    }

    fn side(self) -> TypeRef {
        match self {
            Self::Unit => TypeRef::Tuple(Vec::new()),
            Self::Parameter => {
                TypeRef::Named(Named::new(Sited::implied(Resolved::Param(Name::from(StdAlias::PARAMETER))), Vec::new()))
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum ErrSide {
    Path(&'static [Word<'static>]),
    Opaque,
}

impl ErrSide {
    fn side(self) -> TypeRef {
        match self {
            Self::Path(words) => {
                TypeRef::Named(Named::new(Sited::implied(Resolved::Std(words.iter().copied().collect())), Vec::new()))
            }
            Self::Opaque => TypeRef::Opaque,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct StdAlias {
    alias: &'static [Word<'static>],
    ok: OkSide,
    err: ErrSide,
}

impl StdAlias {
    const PARAMETER: Word<'static> = Word::new("T");
    const RESULT: [Word<'static>; 3] = words!["std", "result", "Result"];

    fn declared(self, mut building: Building) -> Building {
        let result = Sited::implied(Resolved::Std(Self::RESULT.into_iter().collect()));
        let target = TypeRef::Named(Named::new(result, vec![self.ok.side(), self.err.side()]));
        building.std_alias(self.alias.iter().copied().collect(), Alias::new(self.ok.parameters(), target));
        building
    }
}

#[derive(Clone, Copy, Debug)]
struct StdGlob {
    module: Word<'static>,
    members: &'static [Member],
}

impl StdGlob {
    fn path(&self) -> ItemPath {
        [StdNames::STD, self.module].map(Name::from).into_iter().collect()
    }

    fn kind(&self, module: &Name, name: &Name) -> Option<Kind> {
        module.matches::<Whole>(&[self.module]).and_then(|_| self.members.iter().find_map(|member| member.kind(name)))
    }

    fn listed(&self, module: &ItemPath) -> Vec<Listed> {
        self.members
            .iter()
            .map(|member| Name::from(member.name))
            .map(|name| Listed { binding: Binding::Std(module.joined(&name)), name })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Listed {
    name: Name,
    binding: Binding,
}

impl Listed {
    pub(super) fn views(&self) -> impl Iterator<Item = Keyed<'_, View<'_>>> {
        Key::every(&self.name).map(|key| Keyed::new(key, View::from(&self.binding)))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Expansion {
    Statement,
    Items,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Table {
    Prelude,
    Derives,
}

impl Table {
    fn names(self) -> &'static [Canonical] {
        match self {
            Self::Prelude => &StdNames::PRELUDE,
            Self::Derives => &StdNames::DERIVES,
        }
    }
}

#[derive(Debug)]
pub(crate) struct StdNames;

impl StdNames {
    const CANONICAL_SEGMENTS: usize = 3;
    const STD: Word<'static> = Word::new("std");
    const STD_ROOTS: [Word<'static>; 3] = words!["std", "core", "alloc"];
    const PRELUDE: [Canonical; 37] = [
        Canonical::new(Word::new("marker"), Word::new("Copy"), Kind::Trait),
        Canonical::new(Word::new("marker"), Word::new("Send"), Kind::Trait),
        Canonical::new(Word::new("marker"), Word::new("Sized"), Kind::Trait),
        Canonical::new(Word::new("marker"), Word::new("Sync"), Kind::Trait),
        Canonical::new(Word::new("marker"), Word::new("Unpin"), Kind::Trait),
        Canonical::new(Word::new("ops"), Word::new("Drop"), Kind::Trait),
        Canonical::new(Word::new("ops"), Word::new("Fn"), Kind::Trait),
        Canonical::new(Word::new("ops"), Word::new("FnMut"), Kind::Trait),
        Canonical::new(Word::new("ops"), Word::new("FnOnce"), Kind::Trait),
        Canonical::new(Word::new("ops"), Word::new("AsyncFn"), Kind::Trait),
        Canonical::new(Word::new("ops"), Word::new("AsyncFnMut"), Kind::Trait),
        Canonical::new(Word::new("ops"), Word::new("AsyncFnOnce"), Kind::Trait),
        Canonical::new(Word::new("boxed"), Word::new("Box"), Kind::Type),
        Canonical::new(Word::new("borrow"), Word::new("ToOwned"), Kind::Trait),
        Canonical::new(Word::new("clone"), Word::new("Clone"), Kind::Trait),
        Canonical::new(Word::new("cmp"), Word::new("PartialEq"), Kind::Trait),
        Canonical::new(Word::new("cmp"), Word::new("PartialOrd"), Kind::Trait),
        Canonical::new(Word::new("cmp"), Word::new("Eq"), Kind::Trait),
        Canonical::new(Word::new("cmp"), Word::new("Ord"), Kind::Trait),
        Canonical::new(Word::new("convert"), Word::new("AsRef"), Kind::Trait),
        Canonical::new(Word::new("convert"), Word::new("AsMut"), Kind::Trait),
        Canonical::new(Word::new("convert"), Word::new("Into"), Kind::Trait),
        Canonical::new(Word::new("convert"), Word::new("From"), Kind::Trait),
        Canonical::new(Word::new("convert"), Word::new("TryFrom"), Kind::Trait),
        Canonical::new(Word::new("convert"), Word::new("TryInto"), Kind::Trait),
        Canonical::new(Word::new("default"), Word::new("Default"), Kind::Trait),
        Canonical::new(Word::new("iter"), Word::new("Iterator"), Kind::Trait),
        Canonical::new(Word::new("iter"), Word::new("Extend"), Kind::Trait),
        Canonical::new(Word::new("iter"), Word::new("IntoIterator"), Kind::Trait),
        Canonical::new(Word::new("iter"), Word::new("DoubleEndedIterator"), Kind::Trait),
        Canonical::new(Word::new("iter"), Word::new("ExactSizeIterator"), Kind::Trait),
        Canonical::new(Word::new("iter"), Word::new("FromIterator"), Kind::Trait),
        Canonical::new(Word::new("option"), Word::new("Option"), Kind::Type),
        Canonical::new(Word::new("result"), Word::new("Result"), Kind::Type),
        Canonical::new(Word::new("string"), Word::new("String"), Kind::Type),
        Canonical::new(Word::new("string"), Word::new("ToString"), Kind::Trait),
        Canonical::new(Word::new("vec"), Word::new("Vec"), Kind::Type),
    ];
    const DERIVES: [Canonical; 9] = [
        Canonical::new(Word::new("clone"), Word::new("Clone"), Kind::Trait),
        Canonical::new(Word::new("marker"), Word::new("Copy"), Kind::Trait),
        Canonical::new(Word::new("fmt"), Word::new("Debug"), Kind::Trait),
        Canonical::new(Word::new("default"), Word::new("Default"), Kind::Trait),
        Canonical::new(Word::new("cmp"), Word::new("Eq"), Kind::Trait),
        Canonical::new(Word::new("hash"), Word::new("Hash"), Kind::Trait),
        Canonical::new(Word::new("cmp"), Word::new("Ord"), Kind::Trait),
        Canonical::new(Word::new("cmp"), Word::new("PartialEq"), Kind::Trait),
        Canonical::new(Word::new("cmp"), Word::new("PartialOrd"), Kind::Trait),
    ];
    const BEYOND_PRELUDE: [Canonical; 1] = [Canonical::new(Word::new("any"), Word::new("Any"), Kind::Trait)];
    const PRIMITIVES: [PrimitiveName; 17] = [
        PrimitiveName::new(Word::new("bool"), Primitive::Bool),
        PrimitiveName::new(Word::new("char"), Primitive::Char),
        PrimitiveName::new(Word::new("str"), Primitive::Str),
        PrimitiveName::new(Word::new("i8"), Primitive::Integer),
        PrimitiveName::new(Word::new("i16"), Primitive::Integer),
        PrimitiveName::new(Word::new("i32"), Primitive::Integer),
        PrimitiveName::new(Word::new("i64"), Primitive::Integer),
        PrimitiveName::new(Word::new("i128"), Primitive::Integer),
        PrimitiveName::new(Word::new("isize"), Primitive::Integer),
        PrimitiveName::new(Word::new("u8"), Primitive::Integer),
        PrimitiveName::new(Word::new("u16"), Primitive::Integer),
        PrimitiveName::new(Word::new("u32"), Primitive::Integer),
        PrimitiveName::new(Word::new("u64"), Primitive::Integer),
        PrimitiveName::new(Word::new("u128"), Primitive::Integer),
        PrimitiveName::new(Word::new("usize"), Primitive::Integer),
        PrimitiveName::new(Word::new("f32"), Primitive::Float),
        PrimitiveName::new(Word::new("f64"), Primitive::Float),
    ];
    const STD_ALIASES: [StdAlias; 3] = [
        StdAlias {
            alias: &words!["std", "fmt", "Result"],
            ok: OkSide::Unit,
            err: ErrSide::Path(&words!["std", "fmt", "Error"]),
        },
        StdAlias {
            alias: &words!["std", "io", "Result"],
            ok: OkSide::Parameter,
            err: ErrSide::Path(&words!["std", "io", "Error"]),
        },
        StdAlias { alias: &words!["std", "thread", "Result"], ok: OkSide::Parameter, err: ErrSide::Opaque },
    ];
    const STATEMENT_MACROS: [Word<'static>; 20] = words![
        "assert",
        "assert_eq",
        "assert_ne",
        "debug_assert",
        "debug_assert_eq",
        "debug_assert_ne",
        "println",
        "print",
        "eprintln",
        "eprint",
        "format",
        "write",
        "writeln",
        "panic",
        "todo",
        "unimplemented",
        "unreachable",
        "vec",
        "matches",
        "dbg",
    ];
    const STD_GLOBS: [StdGlob; 4] = [
        StdGlob {
            module: Word::new("fmt"),
            members: &[
                Member::new(Word::new("Result"), Kind::Type),
                Member::new(Word::new("Alignment"), Kind::Type),
                Member::new(Word::new("Arguments"), Kind::Type),
                Member::new(Word::new("Binary"), Kind::Trait),
                Member::new(Word::new("Debug"), Kind::Trait),
                Member::new(Word::new("DebugList"), Kind::Type),
                Member::new(Word::new("DebugMap"), Kind::Type),
                Member::new(Word::new("DebugSet"), Kind::Type),
                Member::new(Word::new("DebugStruct"), Kind::Type),
                Member::new(Word::new("DebugTuple"), Kind::Type),
                Member::new(Word::new("Display"), Kind::Trait),
                Member::new(Word::new("Error"), Kind::Type),
                Member::new(Word::new("Formatter"), Kind::Type),
                Member::new(Word::new("LowerExp"), Kind::Trait),
                Member::new(Word::new("LowerHex"), Kind::Trait),
                Member::new(Word::new("Octal"), Kind::Trait),
                Member::new(Word::new("Pointer"), Kind::Trait),
                Member::new(Word::new("UpperExp"), Kind::Trait),
                Member::new(Word::new("UpperHex"), Kind::Trait),
                Member::new(Word::new("Write"), Kind::Trait),
            ],
        },
        StdGlob {
            module: Word::new("io"),
            members: &[
                Member::new(Word::new("Result"), Kind::Type),
                Member::new(Word::new("BufRead"), Kind::Trait),
                Member::new(Word::new("BufReader"), Kind::Type),
                Member::new(Word::new("BufWriter"), Kind::Type),
                Member::new(Word::new("Cursor"), Kind::Type),
                Member::new(Word::new("Error"), Kind::Type),
                Member::new(Word::new("ErrorKind"), Kind::Type),
                Member::new(Word::new("Read"), Kind::Trait),
                Member::new(Word::new("Seek"), Kind::Trait),
                Member::new(Word::new("SeekFrom"), Kind::Type),
                Member::new(Word::new("Write"), Kind::Trait),
            ],
        },
        StdGlob {
            module: Word::new("hash"),
            members: &[
                Member::new(Word::new("BuildHasher"), Kind::Trait),
                Member::new(Word::new("BuildHasherDefault"), Kind::Type),
                Member::new(Word::new("DefaultHasher"), Kind::Type),
                Member::new(Word::new("Hash"), Kind::Trait),
                Member::new(Word::new("Hasher"), Kind::Trait),
                Member::new(Word::new("RandomState"), Kind::Type),
            ],
        },
        StdGlob {
            module: Word::new("cmp"),
            members: &[
                Member::new(Word::new("Eq"), Kind::Trait),
                Member::new(Word::new("Ord"), Kind::Trait),
                Member::new(Word::new("PartialEq"), Kind::Trait),
                Member::new(Word::new("PartialOrd"), Kind::Trait),
                Member::new(Word::new("Ordering"), Kind::Type),
                Member::new(Word::new("Reverse"), Kind::Type),
            ],
        },
    ];

    pub(crate) fn aliased(building: Building) -> Building {
        Self::STD_ALIASES.iter().fold(building, |building, alias| alias.declared(building))
    }

    pub(super) fn resolved(path: ItemPath) -> Resolved {
        Kind::resolved(Self::listed_kind(&path), path)
    }

    fn listed_kind(path: &ItemPath) -> Option<Kind> {
        let [_, module, name] = <&[Name; Self::CANONICAL_SEGMENTS]>::try_from(path.as_ref()).ok()?;
        Self::kind(module, name)
    }

    fn kind(module: &Name, name: &Name) -> Option<Kind> {
        let listed = Self::STD_GLOBS.iter().find_map(|glob| glob.kind(module, name));
        Self::PRELUDE
            .iter()
            .chain(&Self::DERIVES)
            .chain(&Self::BEYOND_PRELUDE)
            .find_map(|canonical| canonical.kind(module, name))
            .or(listed)
    }

    pub(super) fn primitive(only: &Name) -> Option<Primitive> {
        Self::PRIMITIVES.iter().find(|row| only.matches::<Whole>(&[row.word]).is_some()).map(|row| row.primitive)
    }

    pub(super) fn root(first: &Name) -> Option<Binding> {
        first.matches::<Whole>(&Self::STD_ROOTS).map(|_| Binding::Std(ItemPath::from_iter([Name::from(Self::STD)])))
    }

    pub(super) fn canonical(table: Table, only: &Name) -> Option<&'static Canonical> {
        table.names().iter().find(|canonical| only.matches::<Whole>(&[canonical.name]).is_some())
    }

    pub(super) fn listed(module: &ItemPath) -> Option<Vec<Listed>> {
        Self::STD_GLOBS.iter().find(|glob| glob.path() == *module).map(|glob| glob.listed(module))
    }

    pub(super) fn expansion(invoked: &Macro) -> Expansion {
        match invoked.path.get_ident().map(Name::from).and_then(|name| name.matches::<Whole>(&Self::STATEMENT_MACROS)) {
            Some(_) => Expansion::Statement,
            None => Expansion::Items,
        }
    }
}
