use invariants::Ignoring;
use newtype::{Newtype, Wrapping};
use rustc_lexer::{FrontmatterAllowed, Token, TokenKind, strip_shebang, tokenize};
use syn::visit::Visit;

use crate::finding::{Fix, Found, Line, Rule};
use crate::ignored::Ignored;
use crate::source::Source;
use crate::style::{Reads, Style};

#[derive(Debug)]
pub(crate) struct NoComments<'file> {
    source: &'file Source,
}

#[derive(Newtype, Clone, Copy, Debug)]
struct Offset(usize);

#[derive(Debug)]
enum Lexeme {
    Comment(Line),
    Code,
}

#[derive(Debug)]
struct Lexed {
    offset: Offset,
    line: Line,
}

impl NoComments<'_> {
    const FIX: Fix = Fix("delete it and say the same in the names of the code");

    fn lexed(&self, lexed: &mut Lexed, token: &Token) -> Option<Lexeme> {
        let text = self.source.text().as_ref();
        let line = lexed.line;
        let start = lexed.offset.into_inner();
        let end = start + usize::try_from(token.len).ignoring(Ignored::BeyondRange)?;
        let crossed = text.get(start..end).map_or(0, |spelled| spelled.matches('\n').count());
        *lexed = Lexed { offset: Offset::new(end), line: Line::new(line.into_inner() + crossed) };
        Some(match token.kind {
            TokenKind::LineComment { .. } | TokenKind::BlockComment { .. } => Lexeme::Comment(line),
            _ => Lexeme::Code,
        })
    }
}

impl Visit<'_> for NoComments<'_> {}

impl<'file> Style<'file> for NoComments<'file> {
    const RULE: Rule = Rule::NoComments;
    const READS: Reads = Reads::Every;

    fn new(source: &'file Source) -> NoComments<'file> {
        NoComments { source }
    }

    fn found(self) -> Vec<Found> {
        let text = self.source.text().as_ref();
        let shebang = strip_shebang(text).unwrap_or(0);
        let start = Lexed { offset: Offset::new(shebang), line: Line::new(1) };
        let tokens = tokenize(text.get(shebang..).unwrap_or_default(), FrontmatterAllowed::Yes);
        tokens
            .scan(start, |lexed, token| self.lexed(lexed, &token))
            .filter_map(|lexeme| match lexeme {
                Lexeme::Comment(line) => Some(Found::at(line, "comment", NoComments::FIX)),
                Lexeme::Code => None,
            })
            .collect()
    }
}
