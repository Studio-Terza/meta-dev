use invariants::Ignoring;
use rustc_lexer::{FrontmatterAllowed, TokenKind, strip_shebang, tokenize};

use crate::finding::{Findings, Found};
use crate::hooks::{Descent, Hooks};
use crate::ignored::Ignored;
use crate::lint::{Lint, Reads};
use crate::source::Source;

#[derive(Debug)]
pub(crate) struct NoComments;

#[derive(Debug)]
struct Lexed {
    offset: usize,
    line: usize,
}

impl NoComments {
    const FIX: &'static str = "delete it and say the same in the names of the code";
}

impl Hooks for NoComments {
    fn file(&mut self, _: &syn::File) -> Descent {
        Descent::Over
    }
}

impl<'file> Lint<'file> for NoComments {
    const HEADLINE: &'static str = "comments are not allowed in this repository";
    const READS: Reads = Reads::Every;

    type Fact = Findings;

    fn new(_: &'file Source) -> NoComments {
        NoComments
    }

    fn fact(self, source: &'file Source) -> Findings {
        let text = source.text();
        let shebang = strip_shebang(text).unwrap_or(0);
        let start = Lexed { offset: shebang, line: 1 };
        let tokens = tokenize(text.get(shebang..).unwrap_or_default(), FrontmatterAllowed::Yes);
        let found = tokens
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
            .map(|line| Found::at(line, "comment", NoComments::FIX));
        Findings::found(NoComments::HEADLINE, source.shown(), found.collect())
    }
}
