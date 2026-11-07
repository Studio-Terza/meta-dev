mod boundary;

use std::io::Write;
use std::num::NonZeroU8;

use text_lines::Lines;

use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Text};
use boundary::{Attachment, Line};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Verdict<'body> {
    Repair,
    Attached(Attachment<'body>),
    Malformed(Vec<Line<'body>>),
    Missing,
}

impl<'body> Verdict<'body> {
    const FORMS: &'static str = "write a line spec: none, spec: owner <uuid prefix of 8 or more hex digits>, or spec: <project>#D<n> or <project>#L<n> (several comma-separated); a PR that repairs a red main writes the line repair: main instead";

    pub(crate) fn judging(body: &'body Text) -> Self {
        let lines: Vec<Line> = Lines::from(body.as_ref()).map(Line::read).collect();
        let specs: Vec<Line> = lines.iter().copied().filter(|line| line.spec().is_some()).collect();
        let attached = specs.iter().filter_map(|line| line.spec()).find_map(|value| Attachment::try_from(value).ok());
        lines
            .iter()
            .find(|line| **line == Line::Repair)
            .map(|_| Verdict::Repair)
            .or_else(|| attached.map(Verdict::Attached))
            .unwrap_or_else(|| specs.first().map(|_| ()).map_or(Verdict::Missing, |()| Verdict::Malformed(specs)))
    }

    pub(crate) fn reported(&self, out: &mut impl Write) -> Result<(), Failure<'static>> {
        match self {
            Verdict::Repair => Ok(writeln!(
                out,
                "repairing main: the body has the line {}, so the spec line is not checked",
                Line::Repair
            )?),
            Verdict::Attached(attachment) => Ok(writeln!(out, "spec line well formed: spec: {attachment}")?),
            Verdict::Malformed(specs) => Self::malformed(specs, out),
            Verdict::Missing => Self::missing(out),
        }
    }

    fn malformed(specs: &[Line<'_>], out: &mut impl Write) -> Result<(), Failure<'static>> {
        specs.iter().try_for_each(|line| writeln!(out, "spec line malformed: {line}"))?;
        Ok(writeln!(out, "::error::no spec line of the body is well formed; {}", Self::FORMS)?)
    }

    fn missing(out: &mut impl Write) -> Result<(), Failure<'static>> {
        writeln!(out, "spec line missing")?;
        Ok(writeln!(out, "::error::the body has no line starting \"spec: \"; {}", Self::FORMS)?)
    }

    pub(crate) fn exit(&self) -> Exit {
        match self {
            Verdict::Repair | Verdict::Attached(_) => Exit::Success,
            Verdict::Malformed(_) | Verdict::Missing => Exit::Failure(NonZeroU8::MIN),
        }
    }
}
