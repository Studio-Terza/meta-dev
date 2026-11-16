use lint_core::{Fix, Headline, Message, Problem, Site, SourceFile, Violation};
use lint_layout::LineNumber;

use crate::{FileLint, Reads};

#[derive(Debug)]
pub struct OneWalk;

impl OneWalk {
    const AGAIN: Message = Message::new(
        Problem::new("file walked more than once in one run"),
        Fix::new("ride the one sweep with a watcher, a scoped watcher or a lowered fact"),
    );
}

impl FileLint for OneWalk {
    const HEADLINE: Headline =
        Headline::new("syntax is walked once: every lint, the lowering and the facts ride one sweep of each file");
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let again = file
            .walks()
            .repeated()
            .map(|_| Violation::new(Site::new(file.head().path(), LineNumber::FIRST), Self::AGAIN));
        again.into_iter().collect()
    }
}
