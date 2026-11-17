use invariants::{invariant, invariants};

use super::Body;

invariants! {
    enum Invariant {
        EveryPartHasAU32Place =>
            "a body holds at most u32::MAX bytes and each of its parts spans at least one, so every part has a u32 place",
    }
}

impl Body {
    pub fn numbered<Part, Placed>(&self, parts: Vec<Part>, place: impl Fn(u32, Part) -> Placed) -> Vec<Placed> {
        parts
            .into_iter()
            .enumerate()
            .map(|(at, part)| {
                place(
                    u32::try_from(at).unwrap_or_else(|_unnumbered| invariant!(broken Invariant::EveryPartHasAU32Place)),
                    part,
                )
            })
            .collect()
    }
}
