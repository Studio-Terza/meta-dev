use syn::Macro;

use code_model::{Alias, Building, ItemPath, Named, Primitive, Resolved, TypeRef};
use lint_layout::{Name, Whole, Word, words};

use super::binding::Binding;
use super::tables::{Key, Keyed};
use super::view::View;

#[derive(Clone, Copy, Debug)]
pub(super) struct Canonical {
    module: Word<'static>,
    name: Word<'static>,
}

impl Canonical {
    const fn new(module: Word<'static>, name: Word<'static>) -> Canonical {
        Canonical { module, name }
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
    const fn new(word: Word<'static>, primitive: Primitive) -> PrimitiveName {
        PrimitiveName { word, primitive }
    }
}

#[derive(Clone, Copy, Debug)]
enum OkSide {
    Unit,
    Parameter,
}

impl OkSide {
    fn parameters(self) -> Vec<Name> {
        match self {
            OkSide::Unit => Vec::new(),
            OkSide::Parameter => vec![Name::from(StdAlias::PARAMETER)],
        }
    }

    fn side(self) -> TypeRef {
        match self {
            OkSide::Unit => TypeRef::Tuple(Vec::new()),
            OkSide::Parameter => {
                TypeRef::Named(Named::new(Resolved::Param(Name::from(StdAlias::PARAMETER)), Vec::new()))
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
            ErrSide::Path(words) => {
                TypeRef::Named(Named::new(Resolved::Std(words.iter().copied().collect()), Vec::new()))
            }
            ErrSide::Opaque => TypeRef::Opaque,
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
        let result = Resolved::Std(Self::RESULT.into_iter().collect());
        let target = TypeRef::Named(Named::new(result, vec![self.ok.side(), self.err.side()]));
        building.std_alias(self.alias.iter().copied().collect(), Alias::new(self.ok.parameters(), target));
        building
    }
}

#[derive(Clone, Copy, Debug)]
struct StdGlob {
    module: Word<'static>,
    names: &'static [Word<'static>],
}

impl StdGlob {
    fn path(&self) -> ItemPath {
        [StdNames::STD, self.module].map(Name::from).into_iter().collect()
    }

    fn listed(&self, module: &ItemPath) -> Vec<Listed> {
        self.names
            .iter()
            .map(|word| Name::from(*word))
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
            Table::Prelude => &StdNames::PRELUDE,
            Table::Derives => &StdNames::DERIVES,
        }
    }
}

#[derive(Debug)]
pub(crate) struct StdNames;

impl StdNames {
    const STD: Word<'static> = Word::new("std");
    const STD_ROOTS: [Word<'static>; 3] = words!["std", "core", "alloc"];
    const PRELUDE: [Canonical; 37] = [
        Canonical::new(Word::new("marker"), Word::new("Copy")),
        Canonical::new(Word::new("marker"), Word::new("Send")),
        Canonical::new(Word::new("marker"), Word::new("Sized")),
        Canonical::new(Word::new("marker"), Word::new("Sync")),
        Canonical::new(Word::new("marker"), Word::new("Unpin")),
        Canonical::new(Word::new("ops"), Word::new("Drop")),
        Canonical::new(Word::new("ops"), Word::new("Fn")),
        Canonical::new(Word::new("ops"), Word::new("FnMut")),
        Canonical::new(Word::new("ops"), Word::new("FnOnce")),
        Canonical::new(Word::new("ops"), Word::new("AsyncFn")),
        Canonical::new(Word::new("ops"), Word::new("AsyncFnMut")),
        Canonical::new(Word::new("ops"), Word::new("AsyncFnOnce")),
        Canonical::new(Word::new("boxed"), Word::new("Box")),
        Canonical::new(Word::new("borrow"), Word::new("ToOwned")),
        Canonical::new(Word::new("clone"), Word::new("Clone")),
        Canonical::new(Word::new("cmp"), Word::new("PartialEq")),
        Canonical::new(Word::new("cmp"), Word::new("PartialOrd")),
        Canonical::new(Word::new("cmp"), Word::new("Eq")),
        Canonical::new(Word::new("cmp"), Word::new("Ord")),
        Canonical::new(Word::new("convert"), Word::new("AsRef")),
        Canonical::new(Word::new("convert"), Word::new("AsMut")),
        Canonical::new(Word::new("convert"), Word::new("Into")),
        Canonical::new(Word::new("convert"), Word::new("From")),
        Canonical::new(Word::new("convert"), Word::new("TryFrom")),
        Canonical::new(Word::new("convert"), Word::new("TryInto")),
        Canonical::new(Word::new("default"), Word::new("Default")),
        Canonical::new(Word::new("iter"), Word::new("Iterator")),
        Canonical::new(Word::new("iter"), Word::new("Extend")),
        Canonical::new(Word::new("iter"), Word::new("IntoIterator")),
        Canonical::new(Word::new("iter"), Word::new("DoubleEndedIterator")),
        Canonical::new(Word::new("iter"), Word::new("ExactSizeIterator")),
        Canonical::new(Word::new("iter"), Word::new("FromIterator")),
        Canonical::new(Word::new("option"), Word::new("Option")),
        Canonical::new(Word::new("result"), Word::new("Result")),
        Canonical::new(Word::new("string"), Word::new("String")),
        Canonical::new(Word::new("string"), Word::new("ToString")),
        Canonical::new(Word::new("vec"), Word::new("Vec")),
    ];
    const DERIVES: [Canonical; 9] = [
        Canonical::new(Word::new("clone"), Word::new("Clone")),
        Canonical::new(Word::new("marker"), Word::new("Copy")),
        Canonical::new(Word::new("fmt"), Word::new("Debug")),
        Canonical::new(Word::new("default"), Word::new("Default")),
        Canonical::new(Word::new("cmp"), Word::new("Eq")),
        Canonical::new(Word::new("hash"), Word::new("Hash")),
        Canonical::new(Word::new("cmp"), Word::new("Ord")),
        Canonical::new(Word::new("cmp"), Word::new("PartialEq")),
        Canonical::new(Word::new("cmp"), Word::new("PartialOrd")),
    ];
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
            names: &words![
                "Result",
                "Alignment",
                "Arguments",
                "Binary",
                "Debug",
                "DebugList",
                "DebugMap",
                "DebugSet",
                "DebugStruct",
                "DebugTuple",
                "Display",
                "Error",
                "Formatter",
                "LowerExp",
                "LowerHex",
                "Octal",
                "Pointer",
                "UpperExp",
                "UpperHex",
                "Write",
            ],
        },
        StdGlob {
            module: Word::new("io"),
            names: &words![
                "Result",
                "BufRead",
                "BufReader",
                "BufWriter",
                "Cursor",
                "Error",
                "ErrorKind",
                "Read",
                "Seek",
                "SeekFrom",
                "Write",
            ],
        },
        StdGlob {
            module: Word::new("hash"),
            names: &words!["BuildHasher", "BuildHasherDefault", "DefaultHasher", "Hash", "Hasher", "RandomState"],
        },
        StdGlob {
            module: Word::new("cmp"),
            names: &words!["Eq", "Ord", "PartialEq", "PartialOrd", "Ordering", "Reverse"],
        },
    ];

    pub(crate) fn aliased(building: Building) -> Building {
        Self::STD_ALIASES.iter().fold(building, |building, alias| alias.declared(building))
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
