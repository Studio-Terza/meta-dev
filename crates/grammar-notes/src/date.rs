use std::str::FromStr;

use grammar::logos::Logos;
use newtype::{Arbitrary, Checked, Display, Newtype};

use grammar_unspelled::Unspelled;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling {
    #[regex(r"[0-9]{4}-[0-9]{2}-[0-9]{2}")]
    Day,
}

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
#[newtype(checked, strategy = "[0-9]{4}-[0-9]{2}-[0-9]{2}")]
pub struct Date(String);

impl Checked for Date {
    type Error = Unspelled;

    fn checked(day: &String) -> Result<&String, Unspelled> {
        let spelled: Vec<Result<Spelling, ()>> = Spelling::lexer(day).collect();
        <[Result<Spelling, ()>; 1]>::try_from(spelled)
            .ok()
            .and_then(|[only]| only.ok())
            .map(|_day| day)
            .ok_or(Unspelled)
    }
}

impl FromStr for Date {
    type Err = Unspelled;

    fn from_str(day: &str) -> Result<Date, Unspelled> {
        Date::try_new(day.to_owned())
    }
}
