use std::marker::PhantomData;
use std::str::FromStr;

use newtype::Stored;

use crate::cited::Cited;
use crate::output::{Decimal, Field, Kind, Numbered, Token, Unwrapping};

#[derive(Debug)]
pub struct Word<T>(PhantomData<T>);

#[derive(Debug)]
pub struct Phrase<T>(PhantomData<T>);

#[derive(Debug)]
pub struct Id<T>(PhantomData<T>);

#[derive(Debug)]
pub struct Checked<T>(PhantomData<T>);

#[derive(Debug)]
pub struct Bare<T>(PhantomData<T>);

#[derive(Debug)]
pub struct Quoted<T>(PhantomData<T>);

#[derive(Debug)]
pub struct Integer<T>(PhantomData<T>);

impl<T: FromStr> Field for Word<T> {
    type Value = T;

    fn taken(token: Token) -> Result<T, Token> {
        token.parsed(Kind::Bare)
    }
}

impl<T: FromStr> Field for Phrase<T> {
    type Value = T;

    fn taken(token: Token) -> Result<T, Token> {
        token.parsed(Kind::Quoted)
    }
}

impl<T: FromStr> Field for Id<T> {
    type Value = T;

    fn taken(token: Token) -> Result<T, Token> {
        token.spelled::<Numbered>().and_then(|numbered| numbered.parsed(Kind::Bare))
    }
}

impl<T: FromStr> Field for Checked<T> {
    type Value = Result<T, T::Err>;

    fn taken(token: Token) -> Result<Result<T, T::Err>, Token> {
        token.spelled::<Numbered>().and_then(|numbered| numbered.expected(Kind::Bare)).map(|text| text.parse())
    }
}

impl<T: Stored<Kind: Unwrapping<T>>> Field for Bare<T> {
    type Value = T;

    fn taken(token: Token) -> Result<T, Token> {
        token.wrapped(Kind::Bare)
    }
}

impl<T: Stored<Kind: Unwrapping<T>>> Field for Quoted<T> {
    type Value = T;

    fn taken(token: Token) -> Result<T, Token> {
        token.wrapped(Kind::Quoted)
    }
}

impl<T: TryFrom<u64>> Field for Integer<T> {
    type Value = T;

    fn taken(token: Token) -> Result<T, Token> {
        let kept = token.clone();
        token
            .spelled::<Decimal>()
            .and_then(|digits| digits.parsed::<u64>(Kind::Bare))
            .and_then(|number| T::try_from(number).map_err(|_large| kept))
    }
}

impl Field for Cited {
    type Value = Cited;

    fn taken(token: Token) -> Result<Cited, Token> {
        let kept = token.clone();
        token.expected(Kind::Cited).and_then(|text| Cited::read(&text).ok_or(kept))
    }
}
