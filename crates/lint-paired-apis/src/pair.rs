use syn::Ident;

use lint_layout::{Name, Prefix};

#[derive(Debug)]
pub struct Pair {
    opening: &'static str,
    closing: &'static str,
}

impl Pair {
    pub const ALL: [Pair; 6] = [
        Pair { opening: "start", closing: "stop" },
        Pair { opening: "lock", closing: "unlock" },
        Pair { opening: "open", closing: "close" },
        Pair { opening: "claim", closing: "release" },
        Pair { opening: "acquire", closing: "release" },
        Pair { opening: "begin", closing: "end" },
    ];

    pub fn partner(&self, method: &Ident) -> Option<Name> {
        let name = Name::from(method);
        let rest = name.spelled().get(self.opening.len()..).filter(|_| name.matches::<Prefix>(&[self.opening]))?;
        Some(Name::from([self.closing, rest].concat().as_str()))
    }
}
