use super::Body;

impl Body {
    const NUMBERED: &'static str =
        "a body holds at most u32::MAX bytes and each of its parts spans at least one, so every part has a u32 place";

    pub fn numbered<Part, Placed>(&self, parts: Vec<Part>, place: impl Fn(u32, Part) -> Placed) -> Vec<Placed> {
        parts
            .into_iter()
            .enumerate()
            .map(|(at, part)| {
                place(u32::try_from(at).unwrap_or_else(|_unnumbered| unreachable!("{}", Self::NUMBERED)), part)
            })
            .collect()
    }
}
