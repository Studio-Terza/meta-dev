use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Owner(&'static str);

impl Display for Owner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Key<'text>(&'text str);

impl<'text> From<&'text str> for Key<'text> {
    fn from(text: &'text str) -> Self {
        Self(text)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Refusal<'text> {
    key: Key<'text>,
    owner: Owner,
}

impl Display for Refusal<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "`{}` is a method of {}, and an accessor cannot take that name because it would shadow that method",
            self.key.0, self.owner
        )
    }
}

#[derive(Debug)]
pub struct Reserved;

impl Reserved {
    pub const NAMES: [(&'static str, Owner); 31] = [
        ("name", Owner("Named")),
        ("clone", Owner("Clone")),
        ("clone_from", Owner("Clone")),
        ("eq", Owner("PartialEq")),
        ("ne", Owner("PartialEq")),
        ("partial_cmp", Owner("PartialOrd")),
        ("lt", Owner("PartialOrd")),
        ("le", Owner("PartialOrd")),
        ("gt", Owner("PartialOrd")),
        ("ge", Owner("PartialOrd")),
        ("cmp", Owner("Ord")),
        ("max", Owner("Ord")),
        ("min", Owner("Ord")),
        ("clamp", Owner("Ord")),
        ("hash", Owner("Hash")),
        ("hash_slice", Owner("Hash")),
        ("fmt", Owner("Debug and Display")),
        ("to_string", Owner("ToString")),
        ("from_str", Owner("FromStr")),
        ("from", Owner("From")),
        ("into", Owner("Into")),
        ("try_from", Owner("TryFrom")),
        ("try_into", Owner("TryInto")),
        ("borrow", Owner("Borrow")),
        ("borrow_mut", Owner("BorrowMut")),
        ("as_ref", Owner("AsRef")),
        ("to_owned", Owner("ToOwned")),
        ("clone_into", Owner("ToOwned")),
        ("type_id", Owner("Any")),
        ("default", Owner("Default")),
        ("VARIANTS", Owner("VariantArray")),
    ];

    #[must_use]
    pub fn owner(key: Key<'_>) -> Option<Owner> {
        Self::NAMES.iter().find_map(|(name, owner)| (*name == key.0).then_some(*owner))
    }

    #[must_use]
    pub fn refusal(key: Key<'_>) -> Option<Refusal<'_>> {
        Self::owner(key).map(|owner| Refusal { key, owner })
    }
}
