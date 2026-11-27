use invariants::Ignoring;
use rustc_lexer::{FrontmatterAllowed, TokenKind, strip_shebang, tokenize};

use crate::finding::{Found, Rule};
use crate::ignored::Ignored;
use crate::source::Source;
use crate::style::{Reads, Style};

#[derive(Debug)]
pub(crate) struct NoComments<'file> {
    source: &'file Source,
}

#[derive(Debug)]
struct Lexed {
    offset: usize,
    line: usize,
}

impl NoComments<'_> {
    const FIX: &'static str = "delete it and say the same in the names of the code";
}

impl<'file> Style<'file> for NoComments<'file> {
    const RULE: Rule = Rule::NoComments;
    const READS: Reads = Reads::Every;

    fn new(source: &'file Source) -> NoComments<'file> {
        NoComments { source }
    }

    fn found(self) -> Vec<Found> {
        let text = self.source.text();
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
                        .then_some(line),
                )
            })
            .flatten()
            .map(|line| Found::at(line, "comment", NoComments::FIX))
            .collect()
    }
}
