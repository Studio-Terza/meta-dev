use std::fmt;

use invariants::invariant;

use super::Body;

#[derive(Clone, Copy, Debug)]
enum Invariant {
    EveryPartHasAU32Place,
}

impl fmt::Display for Invariant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Invariant::EveryPartHasAU32Place => formatter.write_str(
                "a body holds at most u32::MAX bytes and each of its parts spans at least one, so every part has a u32 place",
            ),
        }
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
