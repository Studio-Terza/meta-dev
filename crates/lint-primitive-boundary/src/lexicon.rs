use code_model::{ItemPath, Named, Primitive, Resolved, Sited, TypeRef};
use lint_layout::{Name, Whole, Word, words};

use crate::finding::Finding;
use crate::legacy::Legacy;

#[derive(Clone, Debug)]
pub enum Identity {
    Primitive(Name),
    Owned,
    Wire(ItemPath),
}

impl Identity {
    pub fn rerooted(&self) -> Legacy {
        match self {
            Identity::Primitive(_) => Legacy::Ineligible,
            Identity::Owned | Identity::Wire(_) => Legacy::Eligible,
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

    pub fn legacy(written: &ItemPath) -> Legacy {
        match Lone::within(written) {
            Some(_) => Legacy::Eligible,
            None => Legacy::Ineligible,
        }
    }

    pub fn primitive(self) -> Legacy {
        match self.word.matches::<Whole>(&Lexicon::PRIMITIVE_WORDS) {
            Some(_) => Legacy::Eligible,
            None => Legacy::Ineligible,
        }
    }

    pub fn primitive_or_rerooted(self, identity: &Identity) -> Legacy {
        match self.word.matches::<Whole>(&Lexicon::PRIMITIVE_WORDS) {
            Some(_) => Legacy::Eligible,
            None => identity.rerooted(),
        }
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
    holders: Vec<ItemPath>,
    doors: Vec<ItemPath>,
    string_from: ItemPath,
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
    const CORE_ANY: [Word<'static>; 3] = words!["core", "any", "Any"];
    const STRING_FROM: [Word<'static>; 2] = words!["String", "from"];
    const ANY: [Word<'static>; 3] = words!["std", "any", "Any"];
    const WIRE_CRATES: [&'static [Word<'static>]; 7] = [
        &words!["serde_json", "Value"],
        &words!["toml", "Value"],
        &words!["serde_yaml", "Value"],
        &words!["rust_decimal", "Decimal"],
        &words!["bigdecimal", "BigDecimal"],
        &words!["num_bigint", "BigInt"],
        &words!["num_bigint", "BigUint"],
    ];
    const HOLDERS: [&'static [Word<'static>]; 4] = [
        &words!["std", "boxed", "Box"],
        &words!["std", "sync", "Arc"],
        &words!["std", "rc", "Rc"],
        &words!["std", "borrow", "Cow"],
    ];
    const DOORS: [&'static [Word<'static>]; 6] = [
        &words!["std", "str", "FromStr"],
        &words!["std", "convert", "TryFrom"],
        &words!["std", "fmt", "Display"],
        &words!["std", "convert", "AsRef"],
        &words!["std", "borrow", "Borrow"],
        &words!["std", "ops", "Deref"],
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
            holders: Lexicon::HOLDERS.iter().map(|holder| holder.iter().copied().collect()).collect(),
            doors: Lexicon::DOORS.iter().map(|door| door.iter().copied().collect()).collect(),
            string_from: Lexicon::STRING_FROM.into_iter().collect(),
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

    pub fn shared_str<'named>(&self, named: &'named Named<Sited>) -> Option<&'named Named<Sited>> {
        let resolved: &Resolved = named.as_ref();
        let holder = resolved.std().into_iter().find(|path| self.holders.contains(path));
        let arguments = named.arguments();
        let held = arguments.first().xor(arguments.get(1)).and_then(TypeRef::named).map(AsRef::<Resolved>::as_ref);
        let text = held.and_then(Resolved::primitive).into_iter().find(|primitive| *primitive == Primitive::Str);
        holder.and(text).map(|_| named)
    }

    pub fn string_from(&self, written: &ItemPath) -> Legacy {
        match [written].into_iter().find(|written| **written == self.string_from) {
            Some(_) => Legacy::Eligible,
            None => Legacy::Ineligible,
        }
    }

    pub fn owned_type<'ty>(&self, ty: &'ty TypeRef) -> Option<&'ty TypeRef> {
        let resolved = ty.named().map(AsRef::<Resolved>::as_ref);
        resolved.and_then(Resolved::std).into_iter().find(|path| **path == self.owned).map(|_| ty)
    }

    pub fn door<'path>(&'path self, interface: &Sited) -> Option<&'path ItemPath> {
        AsRef::<Resolved>::as_ref(interface).std().and_then(|path| self.doors.iter().find(|door| *door == path))
    }
}
