mod problem;

use std::fmt;

use super::{Finding, Fix};
pub use problem::Problem;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message {
    problem: Problem,
    fix: Fix,
}

impl Message {
    #[must_use]
    pub const fn new(problem: Problem, fix: Fix) -> Self {
        Self { problem, fix }
    }
}

impl Finding for Message {
    fn fix(&self) -> Fix {
        self.fix
    }
}

impl fmt::Display for Message {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.problem.fmt(formatter)
    }
}
