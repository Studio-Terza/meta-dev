use std::fmt;
use std::num::NonZeroU8;

use grammar_spec_line::{Attachment, Line, Spec};
use text_lines::Lines;

use crate::cli::ignored::Ignored;
use crate::cli::ports::{Exit, Text};
use invariants::Ignoring;

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
        let lines: Vec<Line> = body
            .as_ref()
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .filter_map(|line| line.parse::<Line>().ignoring(Ignored::Unparsed))
            .collect();
        let attached = lines.iter().filter_map(Line::spec).find_map(Spec::attachment).cloned();
        let (repairs, specs): (Vec<Line>, Vec<Line>) = lines.into_iter().partition(|line| *line == Line::Repair);
        repairs
            .first()
            .map(|_| Verdict::Repair)
            .or_else(|| attached.map(Verdict::Attached))
            .unwrap_or_else(|| specs.first().map(|_| ()).map_or(Verdict::Missing, |()| Verdict::Malformed(specs)))
    }

    pub(crate) fn exit(&self) -> Exit {
        match self {
            Verdict::Repair | Verdict::Attached(_) => Exit::Success,
            Verdict::Malformed(_) | Verdict::Missing => Exit::Failure(NonZeroU8::MIN),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Malformed<'specs>(&'specs [Line]);

#[derive(Clone, Copy, Debug)]
struct Missing;

impl fmt::Display for Verdict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::Repair => writeln!(
                formatter,
                "repairing main: the body has the line {}, so the spec line is not checked",
                Line::Repair
            ),
            Verdict::Attached(attachment) => writeln!(formatter, "spec line well formed: spec: {attachment}"),
            Verdict::Malformed(specs) => write!(formatter, "{}", Malformed(specs)),
            Verdict::Missing => write!(formatter, "{Missing}"),
        }
    }
}

impl fmt::Display for Malformed<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|line| writeln!(formatter, "spec line malformed: {line}"))?;
        writeln!(formatter, "::error::no spec line of the body is well formed; {}", Verdict::FORMS)
    }
}

impl fmt::Display for Missing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "spec line missing")?;
        writeln!(formatter, "::error::the body has no line starting \"spec: \"; {}", Verdict::FORMS)
    }
}
