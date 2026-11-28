use std::fmt;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

use pipeline::Measure;
use proc_macro2::Span;

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

    pub(crate) fn in_file(self, headline: &'static str, path: &Arc<Path>) -> Finding {
        Finding { headline, path: Arc::clone(path), found: self }
    }
}

#[derive(Debug)]
pub(crate) struct Finding {
    headline: &'static str,
    path: Arc<Path>,
    found: Found,
}

#[derive(Debug, Measure)]
pub(crate) struct Findings(Vec<Finding>);

impl Findings {
    pub(crate) fn found(headline: &'static str, path: &Arc<Path>, found: Vec<Found>) -> Findings {
        Findings(found.into_iter().map(|each| each.in_file(headline, path)).collect())
    }

    pub(crate) fn first(&self) -> Option<&Finding> {
        self.0.first()
    }

    pub(crate) fn written(mut self, headlines: &[&'static str], err: &mut impl Write) -> io::Result<()> {
        let rank = |finding: &Finding| headlines.iter().position(|headline| *headline == finding.headline);
        self.0.sort_by(|earlier, later| {
            (rank(earlier), &earlier.path, earlier.found.line).cmp(&(rank(later), &later.path, later.found.line))
        });
        self.0.chunk_by(|earlier, later| earlier.headline == later.headline).try_for_each(|group| {
            group.first().map_or(Ok(()), |leading| writeln!(err, "{}:", leading.headline))?;
            group.iter().try_for_each(|finding| {
                let Found { line, problem, fix } = &finding.found;
                writeln!(err, "{}:{line}: {problem}; {fix}", finding.path.display())
            })
        })
    }
}

impl From<Vec<Finding>> for Findings {
    fn from(found: Vec<Finding>) -> Findings {
        Findings(found)
    }
}

impl IntoIterator for Findings {
    type Item = Finding;
    type IntoIter = std::vec::IntoIter<Finding>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
