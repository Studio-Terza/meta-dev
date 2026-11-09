use std::io::Write;
use std::num::NonZeroU8;

use grammar::spec_line::{Attachment, Line, Spec};
use text_lines::Lines;

use crate::cli::command::failure::Failure;
use crate::cli::ports::{Exit, Text};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Verdict {
    Repair,
    Attached(Attachment),
    Malformed(Vec<Line>),
    Missing,
}

impl Verdict {
    const FORMS: &'static str = "write a line spec: none, spec: owner <uuid prefix of 8 or more hex digits>, or spec: <project>#D<n> or <project>#L<n> (several comma-separated); a PR that repairs a red main writes the line repair: main instead";

    pub(crate) fn judging(body: &Text) -> Verdict {
        let lines: Vec<Line> =
            body.as_ref().parse::<Lines>().into_iter().flatten().filter_map(|line| line.parse::<Line>().ok()).collect();
        let attached = lines.iter().filter_map(Line::spec).find_map(Spec::attachment).cloned();
        let (repairs, specs): (Vec<Line>, Vec<Line>) = lines.into_iter().partition(|line| *line == Line::Repair);
        repairs
            .first()
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

    fn malformed(specs: &[Line], out: &mut impl Write) -> Result<(), Failure<'static>> {
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
