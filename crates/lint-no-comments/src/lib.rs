use lint_core::Lint;
use rustc_lexer::{FrontmatterAllowed, TokenKind, strip_shebang, tokenize};

use lint_core::LineNumber;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;

#[derive(Debug)]
pub struct NoComments;

impl Lint for NoComments {
    fn report(files: &[SourceFile]) -> Report {
        Report::new(
            "comments are not allowed in this repository",
            files
                .iter()
                .flat_map(|file| {
                    Self::comment_lines(file.text())
                        .into_iter()
                        .map(move |line| Violation::new(Site::new(file.path(), line), "comment"))
                })
                .collect(),
        )
    }
}

impl NoComments {
    fn comment_lines(source: &str) -> Vec<LineNumber> {
        let shebang = strip_shebang(source).unwrap_or(0);
        tokenize(source.get(shebang..).unwrap_or_default(), FrontmatterAllowed::Yes)
            .scan(shebang, |offset, token| {
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
