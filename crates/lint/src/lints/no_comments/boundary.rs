use invariants::Ignoring;
use rustc_lexer::{FrontmatterAllowed, TokenKind, strip_shebang, tokenize};

use super::NoComments;
use crate::ignored::Ignored;
use crate::terms::Line;

#[derive(Debug)]
struct Lexed {
    offset: usize,
    line: usize,
}

impl NoComments<'_> {
    pub(super) fn commented(text: &str) -> Vec<Line> {
        let shebang = strip_shebang(text).unwrap_or(0);
        let start = Lexed { offset: shebang, line: 1 };
        let tokens = tokenize(text.get(shebang..).unwrap_or_default(), FrontmatterAllowed::Yes);
        tokens
            .scan(start, |lexed, token| {
                let line = lexed.line;
                let end = lexed.offset + usize::try_from(token.len).ignoring(Ignored::BeyondRange)?;
                let crossed = text.get(lexed.offset..end).map_or(0, |spelled| spelled.matches('\n').count());
                *lexed = Lexed { offset: end, line: line + crossed };
                Some(
                    matches!(token.kind, TokenKind::LineComment { .. } | TokenKind::BlockComment { .. })
                        .then_some(Line::numbered(line)),
                )
            })
            .flatten()
            .collect()
    }
}
