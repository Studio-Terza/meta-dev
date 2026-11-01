use lint_layout::{Name, Word};

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

    pub fn partner(&self, method: &Name) -> Option<Name> {
        method.without_prefix(Word::new(self.opening)).map(|rest| rest.prefixed(Word::new(self.closing)))
    }
}
