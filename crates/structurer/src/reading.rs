use std::str::FromStr;

use design_model::Title;
use grammar_directive::{Lexed, Line, Location, Piece};

use crate::misread::Wanted;
use crate::verb::{By, New, Verb};
use crate::{Directive, Edge, Label, Misread, NodeRef, Proposal, SegmentRef};

#[derive(Debug)]
pub(crate) struct Reader<'line> {
    rest: &'line [Lexed],
    end: &'line Lexed,
}

#[derive(Debug)]
pub(crate) struct Call<'line> {
    name: &'line Lexed,
    arguments: Vec<Term<'line>>,
}

#[derive(Debug)]
pub(crate) enum Term<'line> {
    Atom(&'line Lexed),
    Call(Call<'line>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Following {
    Open,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shape {
    Open,
    Close,
    Comma,
    Word,
    Atom,
    Other,
}

impl<'line> Reader<'line> {
    pub(crate) fn over(line: &'line Line) -> Reader<'line> {
        Reader { rest: line.pieces(), end: line.end() }
    }

    pub(crate) fn directive(mut self) -> Result<Directive, Misread> {
        let call = Call::read(&mut self)?;
        let ended = self.rest.first().map_or(Ok(call), |extra| Err(Misread::expected(extra, Wanted::End)))?;
        ended.verb()?.directive(&ended)
    }

    pub(crate) fn next(&mut self) -> &'line Lexed {
        let (first, rest) = self.rest.split_first().unwrap_or((self.end, &[]));
        self.rest = rest;
        first
    }

    pub(crate) fn peek(&self) -> &'line Piece {
        self.rest.first().unwrap_or(self.end).piece()
    }
}

impl<'line> Call<'line> {
    pub(crate) fn read(reader: &mut Reader<'line>) -> Result<Call<'line>, Misread> {
        let name = reader.next();
        let named = name.piece().word().map(|_word| name).ok_or_else(|| Misread::expected(name, Wanted::Directive))?;
        Call::opened(reader, named)
    }

    pub(crate) fn opened(reader: &mut Reader<'line>, name: &'line Lexed) -> Result<Call<'line>, Misread> {
        let open = reader.next();
        let opened = Some(open).filter(|candidate| Shape::shaped(candidate.piece()) == Shape::Open);
        opened.ok_or_else(|| Misread::expected(open, Wanted::Open))?;
        Term::arguments(reader, Vec::new()).map(|arguments| Call { name, arguments })
    }

    pub(crate) fn verb(&self) -> Result<Verb, Misread> {
        Term::Atom(self.name).named(Wanted::Directive)
    }

    pub(crate) fn exactly<'call, Counted: TryFrom<&'call [Term<'line>]>>(
        &'call self,
        verb: Verb,
    ) -> Result<Counted, Misread> {
        Counted::try_from(self.arguments.as_slice())
            .map_err(|_counted| Misread::Arity { at: Location::from(self.name), verb })
    }

    pub(crate) fn named<Word: FromStr>(&self) -> Option<Word> {
        self.name.piece().word().and_then(|word| word.parse().ok())
    }
}

impl<'line> Term<'line> {
    pub(crate) fn arguments(reader: &mut Reader<'line>, read: Vec<Term<'line>>) -> Result<Vec<Term<'line>>, Misread> {
        let arguments: Vec<Term<'line>> = read.into_iter().chain([Term::read(reader)?]).collect();
        let separator = reader.next();
        Shape::shaped(separator.piece()).separated(separator, reader, arguments)
    }

    pub(crate) fn read(reader: &mut Reader<'line>) -> Result<Term<'line>, Misread> {
        let first = reader.next();
        Shape::shaped(first.piece()).term(first, reader)
    }

    pub(crate) fn atom(&self) -> Option<&'line Lexed> {
        match self {
            Term::Atom(lexed) => Some(lexed),
            Term::Call(_) => None,
        }
    }

    pub(crate) fn call(&self) -> Option<&Call<'line>> {
        match self {
            Term::Atom(_) => None,
            Term::Call(call) => Some(call),
        }
    }

    pub(crate) fn located(&self) -> &'line Lexed {
        match self {
            Term::Atom(lexed) => lexed,
            Term::Call(call) => call.name,
        }
    }

    pub(crate) fn named<Name: FromStr>(&self, wanted: Wanted) -> Result<Name, Misread> {
        self.atom()
            .and_then(|lexed| lexed.piece().word())
            .and_then(|word| word.parse().ok())
            .ok_or_else(|| Misread::expected(self.located(), wanted))
    }

    pub(crate) fn node(&self) -> Result<NodeRef, Misread> {
        self.named(Wanted::Node)
    }

    pub(crate) fn replaced_by(&self, by: &Term<'line>) -> Result<Edge<NodeRef>, Misread> {
        let replacing = by.within::<By>(Wanted::By)?;
        replacing.ending(self.node()?, replacing.node()?)
    }

    pub(crate) fn ending<Source: Clone, Target: From<Source> + PartialEq>(
        &self,
        source: Source,
        target: Target,
    ) -> Result<Edge<Source, Target>, Misread> {
        Edge::new(source, target).map_err(|_looped| Misread::expected(self.located(), Wanted::Distinct))
    }

    pub(crate) fn proposal(&self, fresh: Option<Label>) -> Result<Proposal, Misread> {
        self.node().and_then(|refines| self.refined(fresh, refines))
    }

    pub(crate) fn refined(&self, fresh: Option<Label>, refines: NodeRef) -> Result<Proposal, Misread> {
        match fresh {
            None => Ok(Proposal::Anonymous(refines)),
            Some(label) => self.ending(label, refines).map(Proposal::Labelled),
        }
    }

    pub(crate) fn fresh(&self) -> Result<Option<Label>, Misread> {
        let piece = self.atom().map(Lexed::piece);
        let anonymous = piece.and_then(Piece::word).and_then(|word| word.parse::<New>().ok()).map(|New::New| None);
        let labelled = piece.and_then(Piece::fresh).and_then(|label| label.as_ref().parse::<Label>().ok()).map(Some);
        anonymous.or(labelled).ok_or_else(|| Misread::expected(self.located(), Wanted::New))
    }

    pub(crate) fn segment(&self) -> Result<SegmentRef, Misread> {
        self.atom()
            .and_then(|lexed| lexed.piece().cited())
            .map(SegmentRef::from)
            .ok_or_else(|| Misread::expected(self.located(), Wanted::Segment))
    }

    pub(crate) fn title(&self) -> Result<Title, Misread> {
        self.atom()
            .and_then(|lexed| lexed.piece().quoted())
            .map(|quote| Title::new(quote.as_ref().to_owned()))
            .ok_or_else(|| Misread::expected(self.located(), Wanted::Title))
    }

    pub(crate) fn within<Word: FromStr>(&self, wanted: Wanted) -> Result<&Term<'line>, Misread> {
        self.call()
            .filter(|call| call.named::<Word>().is_some())
            .and_then(|call| <&[Term<'line>; 1]>::try_from(call.arguments.as_slice()).ok())
            .map(|[inner]| inner)
            .ok_or_else(|| Misread::expected(self.located(), wanted))
    }
}

impl Shape {
    pub(crate) fn shaped(piece: &Piece) -> Shape {
        match piece {
            Piece::Open => Shape::Open,
            Piece::Close => Shape::Close,
            Piece::Comma => Shape::Comma,
            Piece::Word(_) => Shape::Word,
            Piece::Fresh(_) | Piece::Segment(_) | Piece::Quoted(_) => Shape::Atom,
            Piece::Break | Piece::End | Piece::Stray(_) => Shape::Other,
        }
    }

    pub(crate) fn separated<'line>(
        self,
        separator: &'line Lexed,
        reader: &mut Reader<'line>,
        arguments: Vec<Term<'line>>,
    ) -> Result<Vec<Term<'line>>, Misread> {
        match self {
            Shape::Comma => Term::arguments(reader, arguments),
            Shape::Close => Ok(arguments),
            Shape::Open | Shape::Word | Shape::Atom | Shape::Other => {
                Err(Misread::expected(separator, Wanted::Separator))
            }
        }
    }

    pub(crate) fn term<'line>(self, first: &'line Lexed, reader: &mut Reader<'line>) -> Result<Term<'line>, Misread> {
        match self {
            Shape::Word => Following::peeked(reader.peek()).term(first, reader),
            Shape::Atom => Ok(Term::Atom(first)),
            Shape::Open | Shape::Close | Shape::Comma | Shape::Other => Err(Misread::expected(first, Wanted::Argument)),
        }
    }
}

impl Following {
    pub(crate) fn peeked(piece: &Piece) -> Following {
        match piece {
            Piece::Open => Following::Open,
            Piece::Close
            | Piece::Comma
            | Piece::Break
            | Piece::End
            | Piece::Word(_)
            | Piece::Fresh(_)
            | Piece::Segment(_)
            | Piece::Quoted(_)
            | Piece::Stray(_) => Following::Other,
        }
    }

    pub(crate) fn term<'line>(self, first: &'line Lexed, reader: &mut Reader<'line>) -> Result<Term<'line>, Misread> {
        match self {
            Following::Open => Call::opened(reader, first).map(Term::Call),
            Following::Other => Ok(Term::Atom(first)),
        }
    }
}
