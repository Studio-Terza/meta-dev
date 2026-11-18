use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Shards {
    mutants: usize,
    count: usize,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Matrix(Shards);

#[derive(Clone, Copy, Debug)]
struct Planned {
    index: usize,
    mutants: usize,
}

impl Shards {
    const MUTANTS_EACH: usize = 60;
    const FREE_PLAN_CONCURRENT_JOBS: usize = 20;
    const OTHER_JOBS_OF_A_RUN: usize = 6;
    const MOST: usize = Self::FREE_PLAN_CONCURRENT_JOBS - Self::OTHER_JOBS_OF_A_RUN;

    pub(super) fn for_mutants(mutants: usize) -> Shards {
        Shards { mutants, count: mutants.div_ceil(Self::MUTANTS_EACH).clamp(1, Self::MOST) }
    }

    pub(super) fn matrix(self) -> Matrix {
        Matrix(self)
    }

    fn planned(self) -> impl Iterator<Item = Planned> {
        let even = self.mutants / self.count;
        let left = self.mutants % self.count;
        (0..self.count).map(move |index| Planned { index, mutants: even + usize::from(index < left) })
    }
}

impl Planned {
    const SETUP_SECONDS: usize = 94;
    const SECONDS_PER_MUTANT: usize = 7;
    const MARGIN_NUMERATOR: usize = 3;
    const MARGIN_DENOMINATOR: usize = 2;
    const SECONDS_PER_MINUTE: usize = 60;
    const FLOOR_MINUTES: usize = 45;
    const HOSTED_RUNNER_MINUTES: usize = 360;

    fn minutes(self) -> usize {
        let predicted = Self::SETUP_SECONDS + Self::SECONDS_PER_MUTANT * self.mutants;
        let capped = (predicted * Self::MARGIN_NUMERATOR).div_ceil(Self::MARGIN_DENOMINATOR * Self::SECONDS_PER_MINUTE);
        capped.clamp(Self::FLOOR_MINUTES, Self::HOSTED_RUNNER_MINUTES)
    }
}

impl fmt::Display for Shards {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "mutants: {} to test in {} shards", self.mutants, self.count)
    }
}

impl fmt::Display for Planned {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{{\"index\":{},\"minutes\":{}}}", self.index, self.minutes())
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let listed: Vec<String> = self.0.planned().map(|planned| planned.to_string()).collect();
        write!(formatter, "[{}]", listed.join(","))
    }
}
