use crate::finding::Finding;
use crate::reach::{Reach, Reached};
use lint_core::Site;
use lint_core::Violation;
use lint_layout::UsePath;
use lint_layout::{Name, Whole, Word, words};

#[derive(Clone, Debug)]
pub struct Spelled {
    reach: Reach,
    spelling: Spelling,
}

impl Spelled {
    pub fn read(spelled: &UsePath, reach: Reach) -> Option<Spelled> {
        Spelling::read(spelled).map(|spelling| Spelled { reach, spelling }).or_else(|| {
            Spelling::in_primitive_module(spelled)
                .map(|primitive| Spelled { reach: Reach::Widened, spelling: Spelling::Primitive(primitive) })
        })
    }

    pub fn reached(self, site: Site<'_>, context: Name) -> Reached<'_> {
        Reached::new(self.reach, Violation::new(site, self.spelling.into_finding(context)))
    }
}

#[derive(Clone, Debug)]
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
    const PRIMITIVE_MODULES: [&'static [Word<'static>]; 2] =
        [&words!["core", "primitive"], &words!["std", "primitive"]];
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

    fn read(spelled: &UsePath) -> Option<Spelling> {
        spelled
            .segments()
            .split_first()
            .filter(|(_, rest)| rest.is_empty())
            .map_or_else(
                || {
                    Self::OWNED
                        .iter()
                        .find_map(|owned| spelled.spells(owned))
                        .and_then(|owned| owned.segments().last().cloned())
                },
                |(single, _)| Self::primitive(single),
            )
            .map(Spelling::Primitive)
            .or_else(|| Self::WIRE_TYPES.iter().find_map(|wire| spelled.spells(wire)).cloned().map(Spelling::Wire))
    }

    fn in_primitive_module(spelled: &UsePath) -> Option<Name> {
        spelled.segments().split_last().and_then(|(last, module)| {
            let module: UsePath = module.iter().cloned().collect();
            Self::PRIMITIVE_MODULES
                .iter()
                .find_map(|primitive| module.spells(primitive))
                .and_then(|_| Self::primitive(last))
        })
    }

    fn primitive(name: &Name) -> Option<Name> {
        name.matches::<Whole>(&Self::PRIMITIVES).map(|_| name.clone())
    }

    fn into_finding(self, context: Name) -> Finding {
        match self {
            Spelling::Primitive(primitive) => Finding::BarePrimitive { primitive, context },
            Spelling::Wire(wire) => Finding::WireForm { wire, context },
        }
    }
}
