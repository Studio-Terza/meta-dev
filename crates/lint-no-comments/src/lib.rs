use lint_core::{Headline, Lint};
use lint_file::{FileLint, Reads};
use rustc_lexer::{FrontmatterAllowed, TokenKind, strip_shebang, tokenize};

use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::Report;
use lint_core::Site;
use lint_core::SourceFile;
use lint_core::Violation;
use lint_layout::LineNumber;
use lint_layout::SourceText;

#[derive(Debug)]
pub struct NoComments;

impl Lint for NoComments {
    fn report(files: &[SourceFile]) -> Report<'_> {
        <Self as FileLint>::reported(files)
    }
}

impl FileLint for NoComments {
    const HEADLINE: Headline = Headline::new("comments are not allowed in this repository");
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let lines = Self::comment_lines(file.text()).into_iter();
        lines.map(|line| Violation::new(Site::new(file.head().path(), line), Self::COMMENT)).collect()
    }
}

impl NoComments {
    const COMMENT: Message =
        Message::new(Problem::new("comment"), Fix::new("delete it and say the same in the names of the code"));

    fn comment_lines(text: &SourceText) -> Vec<LineNumber> {
        let source = text.as_ref();
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
