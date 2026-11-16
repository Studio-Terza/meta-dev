use std::collections::BTreeMap;

use equivalence::{Equivalence, Erased};
use lint_layout::{Casing, Name, Whole, Word, words};
use std::ops::Range;

use syntax::{Lexeme, LexemeKind, Lexemes, Named, Quoted, Symbol, TokenTable};

use super::Token;
use crate::shape::Code;
use crate::shape::{Sign, Spelled, Spelling, Spoken};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Spot {
    Named,
    BeforeLiteral,
    Plain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Sort {
    LiteralWord,
    Keyword,
    Lower,
    Other,
}

#[derive(Clone, Debug)]
struct Fact {
    sort: Sort,
    word: Code,
    spelled: String,
}

#[derive(Debug, Default)]
struct Facts(BTreeMap<Symbol, Fact>);

impl Facts {
    const LITERAL_WORDS: [Word<'static>; 2] = words!["true", "false"];
    const KEYWORDS: [Word<'static>; 40] = words![
        "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern", "fn", "for",
        "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", "self", "static",
        "struct", "super", "trait", "type", "unsafe", "use", "where", "while", "bool", "str", "char", "u8", "usize"
    ];

    fn read(table: &TokenTable) -> Facts {
        Facts(table.symbols().map(|symbol| (symbol, Self::learned(table.spelled(symbol)))).collect())
    }

    fn learned(spelled: &str) -> Fact {
        let name = Name::from(spelled);
        let literal = name.matches::<Whole>(&Self::LITERAL_WORDS).map(|_| Sort::LiteralWord);
        let keyword = name.matches::<Whole>(&Self::KEYWORDS).map(|_| Sort::Keyword);
        let lower = Some(name.casing()).filter(|casing| *casing == Casing::Lower).map(|_| Sort::Lower);
        Fact {
            sort: literal.or(keyword).or(lower).unwrap_or(Sort::Other),
            word: Code::word(spelled),
            spelled: spelled.to_owned(),
        }
    }

    fn fact(&self, symbol: Symbol) -> Option<&Fact> {
        self.0.get(&symbol)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Window {
    low: usize,
    high: usize,
}

impl Window {
    const SECOND: usize = 2;

    pub(super) fn new(low: usize, high: usize) -> Window {
        Window { low, high }
    }

    pub(super) fn indices(self) -> Range<usize> {
        self.low..self.high
    }

    pub(super) fn ends(self) -> [usize; 2] {
        [self.low, self.high]
    }

    fn inside(self, lexemes: &[Lexeme], index: Option<usize>) -> Option<LexemeKind> {
        index
            .filter(|found| *found >= self.low)
            .filter(|found| *found < self.high)
            .and_then(|found| lexemes.get(found))
            .map(|lexeme| lexeme.kind())
    }

    fn neighbours(self, lexemes: &[Lexeme], index: usize) -> Neighbours {
        Neighbours {
            earlier: self.inside(lexemes, index.checked_sub(Self::SECOND)),
            before: self.inside(lexemes, index.checked_sub(1)),
            after: self.inside(lexemes, index.checked_add(1)),
            then: self.inside(lexemes, index.checked_add(Self::SECOND)),
        }
    }
}

#[derive(Debug)]
pub(super) struct Classes {
    facts: Facts,
    equivalence: Equivalence,
}

impl Classes {
    pub(super) fn read(table: &TokenTable, equivalence: Equivalence) -> Classes {
        Classes { facts: Facts::read(table), equivalence }
    }

    pub(super) fn token(&self, lexemes: &[Lexeme], index: usize, window: Window) -> Token {
        let spot = window.neighbours(lexemes, index).spot();
        lexemes.get(index).map_or(Token::Dropped, |lexeme| lexeme.kind().read(&Classifying { classes: self, spot }))
    }

    pub(super) fn spelled(&self, lexemes: &[Lexeme], index: usize, window: Window) -> Option<Spelled> {
        let sign = window.neighbours(lexemes, index).sign();
        lexemes.get(index).and_then(|lexeme| lexeme.kind().read(&Speaking { facts: &self.facts, sign }))
    }

    fn literals_erased(&self) -> Option<Erased> {
        self.equivalence.erases(Erased::Literals)
    }
}

#[derive(Debug)]
struct Classifying<'classes> {
    classes: &'classes Classes,
    spot: Spot,
}

impl Classifying<'_> {
    fn renamed(&self, named: Named, sort: Sort, word: &Code) -> Token {
        let every = self.classes.equivalence.erases(Erased::Names);
        let local = self
            .classes
            .equivalence
            .erases(Erased::Locals)
            .filter(|_| sort == Sort::Lower)
            .filter(|_| self.spot != Spot::Named);
        every.or(local).map_or_else(|| Token::Shaped(word.clone()), |_| Token::Slotted(*named.as_ref()))
    }

    fn sorted(&self, named: Named, fact: &Fact) -> Token {
        match fact.sort {
            Sort::LiteralWord => {
                self.classes.literals_erased().map_or(Token::Shaped(fact.word.clone()), |_| Token::Shaped(Code::Erased))
            }
            Sort::Keyword => Token::Shaped(fact.word.clone()),
            Sort::Lower | Sort::Other => self.renamed(named, fact.sort, &fact.word),
        }
    }
}

impl Lexemes for Classifying<'_> {
    type Read = Token;

    fn word(&self, named: Named) -> Token {
        self.classes.facts.fact(*named.as_ref()).map_or(Token::Dropped, |fact| self.sorted(named, fact))
    }

    fn literal(&self, quoted: Quoted) -> Token {
        let code = self.classes.facts.fact(*quoted.as_ref()).map_or(Code::Erased, |fact| fact.word.literal());
        self.classes.literals_erased().map_or(Token::Shaped(code), |_| Token::Shaped(Code::Erased))
    }

    fn mark(&self, mark: char) -> Token {
        let dropped = Some(mark)
            .filter(|found| *found == '-')
            .filter(|_| self.spot == Spot::BeforeLiteral)
            .and(self.classes.literals_erased());
        dropped.map_or(Token::Shaped(Code::Mark(mark)), |_| Token::Dropped)
    }
}

#[derive(Debug)]
struct Speaking<'facts> {
    facts: &'facts Facts,
    sign: Sign,
}

impl Lexemes for Speaking<'_> {
    type Read = Option<Spelled>;

    fn word(&self, named: Named) -> Option<Spelled> {
        let fact = self.facts.fact(*named.as_ref()).filter(|found| found.sort == Sort::LiteralWord)?;
        Some(Spelled::new(self.sign, Spoken::Word(Name::from(fact.spelled.as_str()))))
    }

    fn literal(&self, quoted: Quoted) -> Option<Spelled> {
        let fact = self.facts.fact(*quoted.as_ref())?;
        Some(Spelled::new(self.sign, Spoken::Literal(Spelling::new(fact.spelled.clone()))))
    }

    fn mark(&self, _: char) -> Option<Spelled> {
        None
    }
}

#[derive(Clone, Copy, Debug)]
struct Neighbours {
    earlier: Option<LexemeKind>,
    before: Option<LexemeKind>,
    after: Option<LexemeKind>,
    then: Option<LexemeKind>,
}

impl Neighbours {
    const DOT: LexemeKind = LexemeKind::Mark('.');
    const COLON: LexemeKind = LexemeKind::Mark(':');
    const MINUS: LexemeKind = LexemeKind::Mark('-');
    const CALLED: [LexemeKind; 2] = [LexemeKind::Mark('('), LexemeKind::Mark('!')];

    fn spot(self) -> Spot {
        let literal = self.after.and_then(LexemeKind::quoted).map(|_| Spot::BeforeLiteral);
        self.named().map(|_| Spot::Named).or(literal).unwrap_or(Spot::Plain)
    }

    fn sign(self) -> Sign {
        self.before.filter(|before| *before == Self::MINUS).map_or(Sign::Plus, |_| Sign::Minus)
    }

    fn named(self) -> Option<LexemeKind> {
        self.before
            .filter(|before| *before == Self::DOT)
            .or_else(|| Self::paired(self.earlier, self.before))
            .or_else(|| self.after.filter(|after| Self::CALLED.contains(after)))
            .or_else(|| Self::paired(self.after, self.then))
    }

    fn paired(first: Option<LexemeKind>, second: Option<LexemeKind>) -> Option<LexemeKind> {
        first.filter(|leading| *leading == Self::COLON).and(second).filter(|trailing| *trailing == Self::COLON)
    }
}
