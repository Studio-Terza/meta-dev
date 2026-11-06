use code_model::{ItemPath, Resolved};
use lint_layout::{Name, Whole, Word, words};

use crate::finding::Finding;

#[derive(Clone, Debug)]
pub enum Identity {
    Primitive(Name),
    Owned,
    Wire(ItemPath),
}

impl Identity {
    pub fn direct(self, lone: Option<Lone<'_>>) -> Option<Identity> {
        match lone.and_then(Lone::primitive) {
            Some(_) => Some(self),
            None => self.pathed(),
        }
    }

    pub fn aliased(self, written: &ItemPath) -> Option<Identity> {
        match Lone::within(written) {
            Some(lone) => lone.primitive().map(|_| self),
            None => self.pathed(),
        }
    }

    fn pathed(self) -> Option<Identity> {
        match self {
            Identity::Primitive(_) => None,
            Identity::Owned | Identity::Wire(_) => Some(self),
        }
    }

    pub fn finding(self, context: Name) -> Finding {
        match self {
            Identity::Primitive(primitive) => Finding::BarePrimitive { primitive, context },
            Identity::Owned => Finding::BarePrimitive { primitive: Name::from(Lexicon::STRING), context },
            Identity::Wire(wire) => Finding::WireForm { wire: wire.as_ref().iter().cloned().collect(), context },
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Lone<'path> {
    word: &'path Name,
}

impl<'path> Lone<'path> {
    pub fn within(written: &'path ItemPath) -> Option<Lone<'path>> {
        let segments: &[Name] = written.as_ref();
        segments.first().xor(segments.get(1)).map(|word| Lone { word })
    }

    pub fn primitive(self) -> Option<Lone<'path>> {
        self.word.matches::<Whole>(&Lexicon::PRIMITIVE_WORDS).map(|_| self)
    }

    fn builtin(self) -> Option<Identity> {
        self.word.matches::<Whole>(&Lexicon::BUILTIN_WORDS).map(|_| Identity::Primitive(self.word.clone()))
    }
}

#[derive(Debug)]
struct Wire {
    resolved: Resolved,
    path: ItemPath,
}

impl Wire {
    fn rooted(words: &[Word<'_>], root: fn(ItemPath) -> Resolved) -> Wire {
        let path: ItemPath = words.iter().copied().collect();
        Wire { resolved: root(path.clone()), path }
    }
}

#[derive(Debug)]
pub struct Lexicon {
    owned: ItemPath,
    owned_spellings: Vec<ItemPath>,
    wires: Vec<Wire>,
}

impl Lexicon {
    pub const STRING: Word<'static> = Word::new("String");
    const PRIMITIVE_WORDS: [Word<'static>; 18] = words![
        "bool", "char", "str", "String", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128",
        "isize", "f32", "f64"
    ];
    const OWNED: [Word<'static>; 3] = words!["std", "string", "String"];
    const OWNED_SPELLINGS: [&'static [Word<'static>]; 3] =
        [&words!["String"], &words!["std", "string", "String"], &words!["alloc", "string", "String"]];
    const BUILTIN_WORDS: [Word<'static>; 17] = words![
        "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize",
        "f32", "f64"
    ];
    const ANY: [Word<'static>; 3] = words!["std", "any", "Any"];
    const CORE_ANY: [Word<'static>; 3] = words!["core", "any", "Any"];
    const WIRE_CRATES: [&'static [Word<'static>]; 7] = [
        &words!["serde_json", "Value"],
        &words!["toml", "Value"],
        &words!["serde_yaml", "Value"],
        &words!["rust_decimal", "Decimal"],
        &words!["bigdecimal", "BigDecimal"],
        &words!["num_bigint", "BigInt"],
        &words!["num_bigint", "BigUint"],
    ];
    pub const DOOR_WORDS: [Word<'static>; 6] = words!["FromStr", "TryFrom", "Display", "AsRef", "Borrow", "Deref"];

    pub fn new() -> Lexicon {
        Lexicon {
            owned: Lexicon::OWNED.into_iter().collect(),
            owned_spellings: Lexicon::OWNED_SPELLINGS.iter().map(|owned| owned.iter().copied().collect()).collect(),
            wires: Lexicon::WIRE_CRATES
                .iter()
                .map(|wire| Wire::rooted(wire, Resolved::External))
                .chain([Wire::rooted(&Lexicon::ANY, Resolved::Std), Wire::rooted(&Lexicon::CORE_ANY, Resolved::Std)])
                .collect(),
        }
    }

    pub fn identity(&self, resolved: &Resolved, written: &ItemPath) -> Option<Identity> {
        let primitive = resolved.primitive().and(written.last()).cloned().map(Identity::Primitive);
        let owned = resolved.std().into_iter().find(|path| **path == self.owned).map(|_| Identity::Owned);
        let wire =
            self.wires.iter().find(|wire| wire.resolved == *resolved).map(|wire| Identity::Wire(wire.path.clone()));
        let unresolved = [resolved].into_iter().find(|resolved| **resolved == Resolved::Unknown(written.clone()));
        primitive.or(owned).or(wire).or_else(|| unresolved.and_then(|_| self.spelled(written)))
    }

    fn spelled(&self, written: &ItemPath) -> Option<Identity> {
        let builtin = Lone::within(written).and_then(Lone::builtin);
        let owned = self.owned_spellings.iter().find(|owned| *owned == written).map(|_| Identity::Owned);
        let wire = self.wires.iter().find(|wire| wire.path == *written).map(|wire| Identity::Wire(wire.path.clone()));
        builtin.or(owned).or(wire)
    }
}
