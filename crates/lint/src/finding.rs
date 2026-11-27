use std::fmt;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

use proc_macro2::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Rule {
    NoComments,
    NoFreeFns,
    VecOutParams,
    NamedLifetimes,
    TypeShape,
    ExpressionFlow,
    ManualIteration,
    ProcessStarters,
    PublicSurface,
    PublicApiTests,
    NoTestOnlyApi,
    BoolMethods,
    NoDyn,
}

impl Rule {
    fn headline(self) -> &'static str {
        match self {
            Rule::NoComments => "comments are not allowed in this repository",
            Rule::NoFreeFns => "free functions are not allowed; make them associated functions or methods",
            Rule::VecOutParams => {
                "a &mut Vec parameter lets any caller push without the checks; give the collection a type that owns its rules"
            }
            Rule::NamedLifetimes => "lifetimes are named with a word, never a letter",
            Rule::TypeShape => "types carry their meaning: no bool fields, no tuples in signatures, no String errors",
            Rule::ExpressionFlow => "control flow is an expression; every branch yields the value the function returns",
            Rule::ManualIteration => "iterators are driven by combinators, not by hand",
            Rule::ProcessStarters => "only ports start processes",
            Rule::PublicSurface => "a crate exports exactly what another crate uses",
            Rule::PublicApiTests => "tests go through public APIs",
            Rule::NoTestOnlyApi => "shipped code has no test-only API",
            Rule::BoolMethods => {
                "a function returning bool forgets what it found and makes the caller branch on a flag"
            }
            Rule::NoDyn => {
                "no dyn: every type is known at compile time, so code takes a generic, returns impl Trait or names an enum instead of a trait object"
            }
        }
    }
}

#[derive(Debug)]
pub(crate) struct Found {
    line: usize,
    problem: String,
    fix: &'static str,
}

impl Found {
    pub(crate) fn at(line: usize, problem: impl fmt::Display, fix: &'static str) -> Found {
        Found { line, problem: problem.to_string(), fix }
    }

    pub(crate) fn spanned(span: Span, problem: impl fmt::Display, fix: &'static str) -> Found {
        Found::at(span.start().line, problem, fix)
    }

    pub(crate) fn in_file(self, rule: Rule, path: &Arc<Path>) -> Finding {
        Finding { rule, path: Arc::clone(path), found: self }
    }
}

#[derive(Debug)]
pub(crate) struct Finding {
    rule: Rule,
    path: Arc<Path>,
    found: Found,
}

#[derive(Debug, Default)]
pub(crate) struct Findings(Vec<Finding>);

impl Findings {
    pub(crate) fn written(mut self, err: &mut impl Write) -> io::Result<()> {
        self.0.sort_by(|earlier, later| {
            (earlier.rule, &earlier.path, earlier.found.line).cmp(&(later.rule, &later.path, later.found.line))
        });
        self.0.chunk_by(|earlier, later| earlier.rule == later.rule).try_for_each(|group| {
            group.first().map_or(Ok(()), |leading| writeln!(err, "{}:", leading.rule.headline()))?;
            group.iter().try_for_each(|finding| {
                let Found { line, problem, fix } = &finding.found;
                writeln!(err, "{}:{line}: {problem}; {fix}", finding.path.display())
            })
        })
    }
}

impl Extend<Finding> for Findings {
    fn extend<Found: IntoIterator<Item = Finding>>(&mut self, found: Found) {
        self.0.extend(found);
    }
}

impl FromIterator<Finding> for Findings {
    fn from_iter<Found: IntoIterator<Item = Finding>>(found: Found) -> Findings {
        Findings(found.into_iter().collect())
    }
}
