#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Symbol(u32);

impl From<usize> for Symbol {
    fn from(index: usize) -> Symbol {
        Symbol(u32::try_from(index).unwrap_or(u32::MAX))
    }
}

impl From<Symbol> for usize {
    fn from(symbol: Symbol) -> usize {
        usize::try_from(symbol.0).unwrap_or(usize::MAX)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LexemeKind {
    Word(Named),
    Literal(Quoted),
    Mark(char),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Named(Symbol);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Quoted(Symbol);

impl AsRef<Symbol> for Named {
    fn as_ref(&self) -> &Symbol {
        &self.0
    }
}

impl AsRef<Symbol> for Quoted {
    fn as_ref(&self) -> &Symbol {
        &self.0
    }
}

pub trait Lexemes {
    type Read;

    fn word(&self, named: Named) -> Self::Read;

    fn literal(&self, quoted: Quoted) -> Self::Read;

    fn mark(&self, mark: char) -> Self::Read;
}

impl LexemeKind {
    #[must_use]
    pub fn quoted(self) -> Option<Quoted> {
        match self {
            LexemeKind::Literal(quoted) => Some(quoted),
            LexemeKind::Word(_) | LexemeKind::Mark(_) => None,
        }
    }

    pub fn read<Reader: Lexemes>(self, reader: &Reader) -> Reader::Read {
        match self {
            LexemeKind::Word(named) => reader.word(named),
            LexemeKind::Literal(quoted) => reader.literal(quoted),
            LexemeKind::Mark(mark) => reader.mark(mark),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Kinded {
    Word,
    Literal,
}

impl Kinded {
    pub(crate) fn lexeme(self, symbol: Symbol) -> LexemeKind {
        match self {
            Kinded::Word => LexemeKind::Word(Named(symbol)),
            Kinded::Literal => LexemeKind::Literal(Quoted(symbol)),
        }
    }
}
