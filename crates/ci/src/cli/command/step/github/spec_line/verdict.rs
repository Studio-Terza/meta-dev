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

    pub(crate) fn judging(body: &'body Text) -> Verdict<'body> {
        let lines: Vec<Line> = Lines::from(body.as_ref()).map(Line::read).collect();
        let specs: Vec<Line> = lines.iter().copied().filter(|line| line.spec().is_some()).collect();
        let attached = specs.iter().filter_map(|line| line.spec()).find_map(|value| Attachment::try_from(value).ok());
        match (lines.contains(&Line::Repair), attached, specs.is_empty()) {
            (true, _, _) => Verdict::Repair,
            (false, Some(attachment), _) => Verdict::Attached(attachment),
            (false, None, true) => Verdict::Missing,
            (false, None, false) => Verdict::Malformed(specs),
        }
    }

    pub(crate) fn reported(&self, out: &mut impl Write) -> Result<(), Failure> {
        match self {
            Verdict::Repair => Ok(writeln!(
                out,
                "repairing main: the body has the line {}, so the spec line is not checked",
                Line::Repair
            )?),
            Verdict::Attached(attachment) => Ok(writeln!(out, "spec line well formed: spec: {attachment}")?),
            Verdict::Malformed(specs) => {
                specs.iter().try_for_each(|line| writeln!(out, "spec line malformed: {line}"))?;
                Ok(writeln!(out, "::error::no spec line of the body is well formed; {}", Self::FORMS)?)
            }
            Verdict::Missing => {
                writeln!(out, "spec line missing")?;
                Ok(writeln!(out, "::error::the body has no line starting \"spec: \"; {}", Self::FORMS)?)
            }
        }
    }

    pub(crate) fn exit(&self) -> Exit {
        match self {
            Verdict::Repair | Verdict::Attached(_) => Exit::Success,
            Verdict::Malformed(_) | Verdict::Missing => Exit::Failure(NonZeroU8::MIN),
        }
    }
}
