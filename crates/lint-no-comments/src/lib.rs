use lint_core::{Headline, Lint};
use rustc_lexer::{FrontmatterAllowed, TokenKind, strip_shebang, tokenize};

use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::LineNumber;

#[derive(Debug)]
pub struct NoComments;

impl Lint for NoComments {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            Headline::new("comments are not allowed in this repository"),
            files
                .iter()
                .flat_map(|file| {
                    Self::comment_lines(file.text().as_ref())
                        .into_iter()
                        .map(move |line| Violation::new(Site::new(file.path(), line), "comment"))
                })
                .collect(),
        )
    }
}

impl NoComments {
    fn comment_lines(source: &str) -> Vec<LineNumber> {
        let skipped = strip_shebang(source).unwrap_or(0);
        tokenize(source.get(skipped..).unwrap_or_default(), FrontmatterAllowed::Yes)
            .scan(skipped, |offset, token| {
                let start = *offset;
                *offset += usize::try_from(token.len).ok()?;
                Some((start, token.kind))
            })
            .filter(|(_, kind)| matches!(kind, TokenKind::LineComment { .. } | TokenKind::BlockComment { .. }))
            .map(|(start, _)| {
                LineNumber::from(source.get(..start).map_or(0, |before| before.matches('\n').count()) + 1)
            })
            .collect()
    }
}
