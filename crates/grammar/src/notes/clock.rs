use std::str::FromStr;

use logos::Logos;
use newtype::{Arbitrary, Checked, Display, Newtype};

use crate::unspelled::Unspelled;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Spelling {
    #[regex(r"[Tt ][0-9]{2}:[0-9]{2}(:[0-9]{2}(\.[0-9]+)?)?([Zz]|[+-][0-9]{2}:[0-9]{2})?")]
    Time,
}

#[derive(Newtype, Arbitrary, Display, Clone, Debug, PartialEq, Eq)]
#[newtype(checked, strategy = "[Tt ][0-9]{2}:[0-9]{2}(:[0-9]{2}(\\.[0-9]{1,3})?)?([Zz]|[+-][0-9]{2}:[0-9]{2})?")]
pub struct Clock(String);

impl Checked for Clock {
    type Error = Unspelled;

    fn checked(time: &String) -> Result<&String, Unspelled> {
        let spelled: Vec<Result<Spelling, ()>> = Spelling::lexer(time).collect();
        <[Result<Spelling, ()>; 1]>::try_from(spelled)
            .ok()
            .and_then(|[only]| only.ok())
            .map(|_time| time)
            .ok_or(Unspelled)
    }
}

impl FromStr for Clock {
    type Err = Unspelled;

    fn from_str(time: &str) -> Result<Clock, Unspelled> {
        Clock::try_new(time.to_owned())
    }
}
