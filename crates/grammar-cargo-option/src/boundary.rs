use grammar::logos::Logos;

use grammar_unspelled::Unspelled;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Head<'text> {
    #[regex("--[a-z][a-z-]*", |lexer| lexer.slice().get(2..))]
    Long(&'text str),
    #[regex("-[vqrhV]*p")]
    Package,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Tail<'text> {
    #[regex(r"=[\s\S]*", |lexer| lexer.slice().get(1..), allow_greedy = true)]
    Assigned(&'text str),
    #[regex(r"[^=][\s\S]*", |lexer| lexer.slice(), allow_greedy = true)]
    Joined(&'text str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CargoOption<'text> {
    Long { name: &'text str, value: Option<&'text str> },
    Package { value: Option<&'text str> },
    Plain,
}

impl<'text> CargoOption<'text> {
    const PACKAGE: &'static str = "package";

    #[must_use]
    pub fn long(self) -> Option<&'text str> {
        match self {
            CargoOption::Long { name, .. } => Some(name),
            CargoOption::Package { .. } | CargoOption::Plain => None,
        }
    }

    #[must_use]
    pub fn spec(self, following: Option<&'text str>) -> Option<Result<&'text str, Unspelled>> {
        match self {
            CargoOption::Long { name, value } => {
                (name == Self::PACKAGE).then_some(value.or(following).ok_or(Unspelled))
            }
            CargoOption::Package { value } => Some(value.or(following).ok_or(Unspelled)),
            CargoOption::Plain => None,
        }
    }

    fn led(lead: Head<'text>, value: Option<Result<Tail<'text>, ()>>) -> Option<CargoOption<'text>> {
        match lead {
            Head::Long(name) => Self::long_valued(name, value),
            Head::Package => Self::package_valued(value),
        }
    }

    fn long_valued(name: &'text str, value: Option<Result<Tail<'text>, ()>>) -> Option<CargoOption<'text>> {
        match value {
            None => Some(CargoOption::Long { name, value: None }),
            Some(Ok(Tail::Assigned(assigned))) => Some(CargoOption::Long { name, value: Some(assigned) }),
            Some(Ok(Tail::Joined(_)) | Err(())) => None,
        }
    }

    fn package_valued(value: Option<Result<Tail<'text>, ()>>) -> Option<CargoOption<'text>> {
        match value {
            None => Some(CargoOption::Package { value: None }),
            Some(Ok(Tail::Assigned(attached) | Tail::Joined(attached))) => {
                Some(CargoOption::Package { value: Some(attached) })
            }
            Some(Err(())) => None,
        }
    }
}

#[derive(Debug)]
pub struct Options;

impl Options {
    #[must_use]
    pub fn spelled(text: &str) -> CargoOption<'_> {
        let mut head = Head::lexer(text);
        let lead = head.next().and_then(Result::ok);
        let mut tail = head.morph::<Tail<'_>>();
        let value = tail.next();
        lead.and_then(|lexed| CargoOption::led(lexed, value)).unwrap_or(CargoOption::Plain)
    }
}
