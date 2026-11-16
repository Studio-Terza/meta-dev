use std::collections::HashMap;

use crate::lexeme::{Kinded, LexemeKind, Symbol};
use syntax::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};

#[derive(Debug)]
pub struct Lexed {
    file: syntax::File,
    tokens: TokenTable,
}

impl Lexed {
    pub fn parsed(content: &str) -> syntax::Result<Lexed> {
        let opened = Opening::read(content);
        let stream: TokenStream = opened.rest().parse()?;
        let parsed: syntax::File = syntax::parse::parse2(stream.clone())?;
        let file = syntax::File { shebang: opened.shebang(), ..parsed };
        Ok(Lexed { file, tokens: TokenTable::flattened(stream) })
    }

    #[must_use]
    pub fn into_file(self) -> syntax::File {
        self.file
    }

    pub fn tokens_taken(&mut self) -> TokenTable {
        std::mem::take(&mut self.tokens)
    }
}

#[derive(Debug)]
struct Opening<'content> {
    shebang: Option<&'content str>,
    rest: &'content str,
}

impl<'content> Opening<'content> {
    const BOM: &'static [u8] = "\u{feff}".as_bytes();
    const SHEBANG: &'static [u8] = b"#!";

    fn read(content: &'content str) -> Opening<'content> {
        let marked = Some(content.as_bytes()).filter(|bytes| bytes.get(..Self::BOM.len()) == Some(Self::BOM));
        let unmarked = marked.and_then(|_| content.get(Self::BOM.len()..)).unwrap_or(content);
        let bytes = unmarked.as_bytes();
        let banged = Some(bytes).filter(|found| found.get(..Self::SHEBANG.len()) == Some(Self::SHEBANG));
        let attribute = banged
            .and_then(|found| found.get(Self::SHEBANG.len()..))
            .and_then(|rest| rest.iter().find(|byte| byte.is_ascii_whitespace().then_some(()).is_none()))
            .filter(|first| **first == b'[');
        let line = banged
            .filter(|_| attribute.is_none())
            .map(|found| found.iter().position(|byte| *byte == b'\n').unwrap_or(found.len()));
        line.map_or(Opening { shebang: None, rest: unmarked }, |end| Opening {
            shebang: unmarked.get(..end),
            rest: unmarked.get(end..).unwrap_or_default(),
        })
    }

    fn rest(&self) -> &'content str {
        self.rest
    }

    fn shebang(&self) -> Option<String> {
        self.shebang.map(str::to_owned)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Place {
    line: u32,
    column: u32,
}

impl From<Span> for Place {
    fn from(span: Span) -> Place {
        Place::from(span.start())
    }
}

impl From<syntax::LineColumn> for Place {
    fn from(at: syntax::LineColumn) -> Place {
        Place { line: Offset::counted(at.line), column: Offset::counted(at.column) }
    }
}

impl Place {
    #[must_use]
    pub fn line(self) -> usize {
        usize::try_from(self.line).unwrap_or(usize::MAX)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lexeme {
    kind: LexemeKind,
    place: Place,
}

impl Lexeme {
    #[must_use]
    pub fn kind(self) -> LexemeKind {
        self.kind
    }

    #[must_use]
    pub fn place(self) -> Place {
        self.place
    }
}

#[derive(Debug, Default)]
pub struct TokenTable {
    lexemes: Vec<Lexeme>,
    symbols: Vec<String>,
}

impl TokenTable {
    fn flattened(stream: TokenStream) -> TokenTable {
        let start = Flattening { known: HashMap::new(), table: TokenTable::default() };
        stream.into_iter().fold(start, Flattening::gathered).table
    }

    #[must_use]
    pub fn lexemes(&self) -> &[Lexeme] {
        &self.lexemes
    }

    pub fn symbols(&self) -> impl Iterator<Item = Symbol> + '_ {
        (0..self.symbols.len()).map(Symbol::from)
    }

    #[must_use]
    pub fn spelled(&self, symbol: Symbol) -> &str {
        self.symbols.get(usize::from(symbol)).map_or("", String::as_str)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Spelled {
    Word(Ident),
    Literal(String),
}

impl Spelled {
    fn text(&self) -> String {
        match self {
            Spelled::Word(ident) => ident.to_string(),
            Spelled::Literal(literal) => literal.clone(),
        }
    }
}

#[derive(Debug)]
struct Flattening {
    known: HashMap<Spelled, Symbol>,
    table: TokenTable,
}

impl Flattening {
    fn gathered(self, tree: TokenTree) -> Flattening {
        match tree {
            TokenTree::Ident(ident) => self.pushed(ident.span(), Spelled::Word(ident), Kinded::Word),
            TokenTree::Literal(literal) => {
                self.pushed(literal.span(), Spelled::Literal(literal.to_string()), Kinded::Literal)
            }
            TokenTree::Punct(punct) => self.marked(punct.as_char(), punct.span()),
            TokenTree::Group(group) => self.grouped(group),
        }
    }

    fn grouped(self, group: Group) -> Flattening {
        let marks = Marks::delimiting(group.delimiter());
        let edges = Edges { open: group.span_open(), close: group.span_close() };
        let stream = group.stream();
        drop(group);
        let opened = marks.into_iter().fold(self, |flat, found| flat.marked(found.open, edges.open));
        let inner = stream.into_iter().fold(opened, Flattening::gathered);
        marks.into_iter().fold(inner, |flat, found| flat.marked(found.close, edges.close))
    }

    fn marked(mut self, mark: char, span: Span) -> Flattening {
        self.table.lexemes.push(Lexeme { kind: LexemeKind::Mark(mark), place: Place::from(span) });
        self
    }

    fn pushed(mut self, span: Span, spelled: Spelled, kinded: Kinded) -> Flattening {
        let next = Symbol::from(self.table.symbols.len());
        let symbol = self.known.get(&spelled).copied().unwrap_or(next);
        let fresh = (symbol == next).then_some(spelled);
        self.table.symbols.extend(fresh.as_ref().map(Spelled::text));
        self.known.extend(fresh.map(|new| (new, next)));
        self.table.lexemes.push(Lexeme { kind: kinded.lexeme(symbol), place: Place::from(span) });
        self
    }
}

#[derive(Clone, Copy, Debug)]
struct Edges {
    open: Span,
    close: Span,
}

#[derive(Debug)]
struct Offset;

impl Offset {
    fn counted(count: usize) -> u32 {
        u32::try_from(count).unwrap_or(u32::MAX)
    }
}

#[derive(Clone, Copy, Debug)]
struct Marks {
    open: char,
    close: char,
}

impl Marks {
    fn delimiting(delimiter: Delimiter) -> Option<Marks> {
        match delimiter {
            Delimiter::Parenthesis => Some(Marks { open: '(', close: ')' }),
            Delimiter::Brace => Some(Marks { open: '{', close: '}' }),
            Delimiter::Bracket => Some(Marks { open: '[', close: ']' }),
            Delimiter::None => None,
        }
    }
}
