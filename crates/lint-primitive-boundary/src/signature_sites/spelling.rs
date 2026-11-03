use crate::finding::Finding;
use lint_layout::UsePath;
use lint_layout::{Name, Whole, Word, words};

pub enum Spelling {
    Primitive(Name),
    Wire(UsePath),
}

impl Spelling {
    const PRIMITIVES: [Word<'static>; 18] = words![
        "bool", "char", "str", "String", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128",
        "isize", "f32", "f64"
    ];
    const OWNED: [&'static [Word<'static>]; 2] =
        [&words!["std", "string", "String"], &words!["alloc", "string", "String"]];
    const WIRE_TYPES: [&'static [Word<'static>]; 9] = [
        &words!["serde_json", "Value"],
        &words!["toml", "Value"],
        &words!["serde_yaml", "Value"],
        &words!["rust_decimal", "Decimal"],
        &words!["bigdecimal", "BigDecimal"],
        &words!["num_bigint", "BigInt"],
        &words!["num_bigint", "BigUint"],
        &words!["std", "any", "Any"],
        &words!["core", "any", "Any"],
    ];

    pub fn read(spelled: &UsePath) -> Option<Spelling> {
        Self::primitive(spelled)
            .map(Spelling::Primitive)
            .or_else(|| Self::WIRE_TYPES.iter().find_map(|wire| spelled.spells(wire)).cloned().map(Spelling::Wire))
    }

    pub fn into_finding(self, context: Name) -> Finding {
        match self {
            Spelling::Primitive(primitive) => Finding::BarePrimitive { primitive, context },
            Spelling::Wire(wire) => Finding::WireForm { wire, context },
        }
    }

    fn primitive(spelled: &UsePath) -> Option<Name> {
        match spelled.segments() {
            [single] => single.matches::<Whole>(&Self::PRIMITIVES).map(|_| single.clone()),
            _ => Self::OWNED
                .iter()
                .find_map(|owned| spelled.spells(owned))
                .and_then(|owned| owned.segments().last().cloned()),
        }
    }
}
