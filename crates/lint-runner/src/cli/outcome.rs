use std::fmt;
use std::io::{self, Write};

use lint_layout::Failure;
use lint_reader::Unlocated;

use super::Verdict;
use super::boundary::Rendered;

#[derive(Debug)]
pub enum Outcome<'run> {
    Clean,
    Violations { reports: Vec<Rendered>, failures: &'run [Failure] },
    Unlocated(Unlocated),
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

    pub(crate) fn written(&self, err: &mut impl Write) -> Verdict {
        match self {
            Outcome::Clean => Verdict::Clean,
            Outcome::Violations { reports, failures } => Self::listed(reports, failures, err),
            Outcome::Unlocated(failure) => Self::refused(failure, err),
        }
    }

    pub(crate) fn surveyed(&self, survey: fmt::Arguments<'_>, out: &mut impl Write, err: &mut impl Write) -> Verdict {
        let surveyed = self.survey(survey).iter().try_for_each(|shown| write!(out, "{shown}"));
        let verdict = self.written(err);
        surveyed.map_or_else(|failure| Self::refused(&Unsurveyed(failure), err), |()| verdict)
    }

    fn survey<'args>(&self, survey: fmt::Arguments<'args>) -> Option<fmt::Arguments<'args>> {
        match self {
            Outcome::Unlocated(_) => None,
            Outcome::Clean | Outcome::Violations { .. } => Some(survey),
        }
    }

    fn listed(reports: &[Rendered], failures: &[Failure], err: &mut impl Write) -> Verdict {
        write!(err, "{}{}", Listed(reports), Failed(failures)).ok();
        Verdict::Violations
    }

    fn refused(failure: &impl fmt::Display, err: &mut impl Write) -> Verdict {
        writeln!(err, "{failure}").ok();
        Verdict::Refused
    }
}

struct Listed<'reports>(&'reports [Rendered]);

impl fmt::Display for Listed<'_> {
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

struct Unsurveyed(io::Error);

impl fmt::Display for Unsurveyed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "cannot write the survey: {}", self.0)
    }
}
