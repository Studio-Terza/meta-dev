mod finding;

use std::collections::BTreeSet;

use crate::declared::Declared;
use finding::Finding;
use lint_core::Violation;
use lint_layout::{Name, Word};

#[derive(Debug)]
pub struct Pair {
    opening: Word<'static>,
    closing: Word<'static>,
}

impl Pair {
    pub const ALL: [Self; 6] = [
        Self { opening: Word::new("start"), closing: Word::new("stop") },
        Self { opening: Word::new("lock"), closing: Word::new("unlock") },
        Self { opening: Word::new("open"), closing: Word::new("close") },
        Self { opening: Word::new("claim"), closing: Word::new("release") },
        Self { opening: Word::new("acquire"), closing: Word::new("release") },
        Self { opening: Word::new("begin"), closing: Word::new("end") },
    ];

    pub fn violations<'file>(&self, methods: &[Declared<'file>]) -> Vec<Violation<'file>> {
        let names: BTreeSet<&Name> = methods.iter().map(Declared::method).collect();
        methods
            .iter()
            .filter_map(|method| {
                method
                    .method()
                    .reprefixed(self.opening, self.closing)
                    .filter(|partner| names.contains(partner))
                    .map(|partner| Violation::new(method.location(), Finding::new(method.method(), partner)))
            })
            .collect()
    }
}
