use lint_core::{Fix, Headline, Message, Parsed, Problem, Site, SourceFile, Violation};
use lint_layout::LineNumber;
use lint_sweep::Unwatched;

use crate::{ContextLint, Reads};

#[derive(Debug)]
pub struct OneWalk;

impl OneWalk {
    const AGAIN: Message = Message::new(
        Problem::new("file walked more than once in one run"),
        Fix::new("ride the one sweep with a watcher, a scoped watcher or a lowered fact"),
    );
}

impl ContextLint for OneWalk {
    const HEADLINE: Headline =
        Headline::new("syntax is walked once: every lint, the lowering and the facts ride one sweep of each file");
    const READS: Reads = Reads::Every;

    type Part<'head> = Unwatched;
    type Context<'head> = Unwatched;

    type Gathering<'head> = Unwatched;

    fn gathering(_: Parsed<'_, '_>) -> Unwatched {
        Unwatched
    }

    type Found<'file> = Unwatched;
    type Watched<'file> = Unwatched;

    fn watched(_: &SourceFile) -> Unwatched {
        Unwatched
    }

    fn judged<'file, 'head: 'file>(file: &'file SourceFile, _: Unwatched, _: &Unwatched) -> Vec<Violation<'file>> {
        let again = file
            .walks()
            .repeated()
            .map(|_| Violation::new(Site::new(file.head().path(), LineNumber::FIRST), Self::AGAIN));
        again.into_iter().collect()
    }
}
