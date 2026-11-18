use std::fmt;
use std::io::{self, Write};

use lint_layout::Failure;
use lint_reader::{Described, Unlocated};

use super::Verdict;
use super::boundary::Rendered;

#[derive(Debug)]
pub enum Outcome<'run> {
    Clean,
    Violations { reports: Vec<Rendered>, failures: &'run [Failure] },
    Unlocated(&'run Unlocated),
}

impl<'run> Outcome<'run> {
    #[must_use]
    pub fn from_rendered(reports: Vec<Rendered>, failures: &'run [Failure]) -> Outcome<'run> {
        reports
            .is_empty()
            .then_some(failures)
            .filter(|candidate| candidate.is_empty())
            .map_or(Outcome::Violations { reports, failures }, |_| Outcome::Clean)
    }

    pub(crate) fn located(
        located: &'run Result<Described, Unlocated>,
        reports: Vec<Rendered>,
        failures: &'run [Failure],
    ) -> Outcome<'run> {
        match located {
            Ok(_described) => Outcome::from_rendered(reports, failures),
            Err(unlocated) => Outcome::Unlocated(unlocated),
        }
    }

    pub(crate) fn written(&self, err: &mut impl Write) -> Verdict {
        match self {
            Outcome::Clean => Verdict::Clean,
            Outcome::Violations { reports, failures } => Self::listed(reports, failures, err),
            Outcome::Unlocated(failure) => Verdict::refused(failure, err),
        }
    }

    pub(crate) fn surveyed(&self, survey: &[impl fmt::Display], out: &mut impl Write, err: &mut impl Write) -> Verdict {
        let surveyed = self.survey(survey).iter().try_for_each(|shown| write!(out, "{}", Listed(shown)));
        let verdict = self.written(err);
        Verdict::ran(surveyed.map(|()| verdict).map_err(Unwritten::Output), err)
    }

    fn survey<'shown, Shown>(&self, survey: &'shown [Shown]) -> Option<&'shown [Shown]> {
        match self {
            Outcome::Unlocated(_) => None,
            Outcome::Clean | Outcome::Violations { .. } => Some(survey),
        }
    }

    fn listed(reports: &[Rendered], failures: &[Failure], err: &mut impl Write) -> Verdict {
        let listed = write!(err, "{}{}", Listed(reports), Failed(failures));
        Verdict::ran(listed.map(|()| Verdict::Violations).map_err(Unwritten::Findings), err)
    }
}

struct Listed<'items, Item>(&'items [Item]);

impl<Item: fmt::Display> fmt::Display for Listed<'_, Item> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|report| write!(formatter, "{report}"))
    }
}

struct Failed<'failures>(&'failures [Failure]);

impl fmt::Display for Failed<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|failure| writeln!(formatter, "{failure}"))
    }
}

#[derive(Debug, thiserror::Error)]
enum Unwritten {
    #[error("cannot write the output: {0}")]
    Output(io::Error),
    #[error("cannot write the findings: {0}")]
    Findings(io::Error),
}
