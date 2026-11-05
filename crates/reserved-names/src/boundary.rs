#[derive(Debug)]
pub struct Reserved;

impl Reserved {
    pub const OWN_NAME: &'static str = "name";
    pub const METHODS: [(&'static str, &'static [&'static str]); 19] = [
        ("Clone", &["clone", "clone_from"]),
        ("PartialEq", &["eq", "ne"]),
        ("PartialOrd", &["partial_cmp", "lt", "le", "gt", "ge"]),
        ("Ord", &["cmp", "max", "min", "clamp"]),
        ("Hash", &["hash", "hash_slice"]),
        ("Debug", &["fmt"]),
        ("Display", &["fmt"]),
        ("ToString", &["to_string"]),
        ("FromStr", &["from_str"]),
        ("From", &["from"]),
        ("Into", &["into"]),
        ("TryFrom", &["try_from"]),
        ("TryInto", &["try_into"]),
        ("Borrow", &["borrow"]),
        ("BorrowMut", &["borrow_mut"]),
        ("AsRef", &["as_ref"]),
        ("ToOwned", &["to_owned", "clone_into"]),
        ("Any", &["type_id"]),
        ("Default", &["default"]),
    ];

    #[must_use]
    pub fn owner(key: &str) -> Option<&'static str> {
        Some(Self::OWN_NAME)
            .filter(|own| *own == key)
            .or_else(|| Self::METHODS.iter().find_map(|(owner, names)| names.contains(&key).then_some(*owner)))
    }

    #[must_use]
    pub fn message() -> String {
        let owners = Self::METHODS.iter().map(|(owner, _)| *owner).collect::<Vec<&str>>();
        let (last, rest) = owners.split_last().unwrap_or((&"", &[]));
        format!(
            "an accessor cannot take the name of a method of {} or {last}, or of the derive's own `{}`, because it would shadow that method",
            rest.join(", "),
            Self::OWN_NAME
        )
    }
}
