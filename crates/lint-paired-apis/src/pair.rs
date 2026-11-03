use lint_layout::{Name, Word};

#[derive(Debug)]
pub struct Pair {
    opening: Word<'static>,
    closing: Word<'static>,
}

impl Pair {
    pub const ALL: [Pair; 6] = [
        Pair { opening: Word::new("start"), closing: Word::new("stop") },
        Pair { opening: Word::new("lock"), closing: Word::new("unlock") },
        Pair { opening: Word::new("open"), closing: Word::new("close") },
        Pair { opening: Word::new("claim"), closing: Word::new("release") },
        Pair { opening: Word::new("acquire"), closing: Word::new("release") },
        Pair { opening: Word::new("begin"), closing: Word::new("end") },
    ];

    pub fn partner(&self, method: &Name) -> Option<Name> {
        method.without_prefix(self.opening).map(|rest| rest.prefixed(self.closing))
    }
}
