use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Shards {
    mutants: usize,
    count: usize,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Indices(usize);

impl Shards {
    const MUTANTS_EACH: usize = 60;
    const MOST: usize = 32;

    pub(super) fn for_mutants(mutants: usize) -> Shards {
        Shards { mutants, count: mutants.div_ceil(Self::MUTANTS_EACH).clamp(1, Self::MOST) }
    }

    pub(super) fn indices(self) -> Indices {
        Indices(self.count)
    }
}

impl fmt::Display for Shards {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "mutants: {} to test in {} shards", self.mutants, self.count)
    }
}

impl fmt::Display for Indices {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let listed: Vec<String> = (0..self.0).map(|index| index.to_string()).collect();
        write!(formatter, "[{}]", listed.join(","))
    }
}
