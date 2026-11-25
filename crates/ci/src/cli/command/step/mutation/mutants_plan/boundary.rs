use std::collections::BTreeMap;
use std::fmt;
use std::time::Duration;

use super::super::kept::{Mutant, Spent};

#[derive(Debug, Default)]
pub(super) struct Costs(BTreeMap<String, Duration>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Shards {
    each: Vec<Duration>,
    count: usize,
}

#[derive(Clone, Debug)]
pub(super) struct Matrix(Shards);

#[derive(Clone, Copy, Debug)]
struct Planned {
    index: usize,
    seconds: Duration,
}

impl Costs {
    const JOBS: u32 = 2;
    const UNMEASURED: Duration = Duration::from_secs(11);

    pub(super) fn measured(spent: Vec<Spent>) -> Costs {
        Costs(
            Spent::averaged(spent)
                .into_iter()
                .filter_map(|(package, mean)| Some((package, mean.checked_div(Self::JOBS)?)))
                .collect(),
        )
    }

    pub(super) fn per_mutant(&self, mutant: &Mutant) -> Duration {
        self.0.get(&mutant.package()).copied().unwrap_or(Self::UNMEASURED)
    }

    pub(super) fn unmeasured(mutants: usize) -> Vec<Duration> {
        vec![Self::UNMEASURED; mutants]
    }
}

impl Shards {
    const MUTANTS_EACH: usize = 60;
    const FREE_PLAN_CONCURRENT_JOBS: usize = 20;
    const OTHER_JOBS_OF_A_RUN: usize = 6;
    const MOST: usize = Self::FREE_PLAN_CONCURRENT_JOBS - Self::OTHER_JOBS_OF_A_RUN;

    pub(super) fn for_mutants(each: Vec<Duration>) -> Shards {
        let count = each.len().div_ceil(Self::MUTANTS_EACH).clamp(1, Self::MOST);
        Shards { each, count }
    }

    pub(super) fn matrix(self) -> Matrix {
        Matrix(self)
    }

    fn planned(&self) -> impl Iterator<Item = Planned> + '_ {
        (0..self.count)
            .map(move |index| Planned { index, seconds: self.each.iter().skip(index).step_by(self.count).sum() })
    }
}

impl Planned {
    const SETUP: Duration = Duration::from_secs(94);
    const MARGIN_NUMERATOR: u32 = 3;
    const MARGIN_DENOMINATOR: u32 = 2;
    const SECONDS_PER_MINUTE: u64 = 60;
    const FLOOR_MINUTES: u64 = 45;
    const HOSTED_RUNNER_MINUTES: u64 = 360;

    fn minutes(self) -> u64 {
        let predicted = Self::SETUP.saturating_add(self.seconds);
        let margined = predicted.saturating_mul(Self::MARGIN_NUMERATOR) / Self::MARGIN_DENOMINATOR;
        margined.as_secs().div_ceil(Self::SECONDS_PER_MINUTE).clamp(Self::FLOOR_MINUTES, Self::HOSTED_RUNNER_MINUTES)
    }
}

impl fmt::Display for Shards {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "mutants: {} to test in {} shards", self.each.len(), self.count)
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
