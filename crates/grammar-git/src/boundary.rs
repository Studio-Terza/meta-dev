use grammar::logos::Logos;
use grammar::{Ignoring, Unlexed};
use grammar_lines::Lines;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Piece<'text> {
    #[token("ref: ")]
    Symbolic,
    #[token("gitdir: ")]
    Gitdir,
    #[regex("[0-9a-f]{40}", |lexer| lexer.slice(), priority = 10)]
    Object(&'text str),
    #[regex("[^ ]+", |lexer| lexer.slice(), priority = 1)]
    Word(&'text str),
    #[token(" ")]
    Space,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitFile(Vec<String>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Object(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefName(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Linked(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Common(String);

impl<'text> Piece<'text> {
    fn object(self) -> Option<Object> {
        match self {
            Piece::Object(object) => Some(Object(object.to_owned())),
            Piece::Symbolic | Piece::Gitdir | Piece::Word(_) | Piece::Space => None,
        }
    }

    fn word(self) -> Option<&'text str> {
        match self {
            Piece::Word(word) => Some(word),
            Piece::Symbolic | Piece::Gitdir | Piece::Object(_) | Piece::Space => None,
        }
    }

    fn lexed<const PIECES: usize>(line: &'text str) -> Option<[Piece<'text>; PIECES]> {
        let pieces: Vec<Piece<'text>> =
            Piece::lexer(line).collect::<Result<_, ()>>().ignoring(Unlexed::NotThisGrammar).unwrap_or_default();
        <[Piece<'text>; PIECES]>::try_from(pieces).ignoring(Unlexed::OtherShape)
    }
}

impl GitFile {
    #[must_use]
    pub fn read(text: &str) -> GitFile {
        GitFile(text.parse::<Lines>().into_iter().flatten().collect())
    }

    #[must_use]
    pub fn object(&self) -> Option<Object> {
        self.first().and_then(Piece::lexed::<1>).and_then(|[only]| only.object())
    }

    #[must_use]
    pub fn symbolic(&self) -> Option<RefName> {
        self.first()
            .and_then(Piece::lexed::<2>)
            .filter(|[mark, _name]| *mark == Piece::Symbolic)
            .and_then(|[_mark, name]| name.word())
            .map(|name| RefName(name.to_owned()))
    }

    #[must_use]
    pub fn linked(&self) -> Option<Linked> {
        self.first()
            .and_then(Piece::lexed::<2>)
            .filter(|[mark, _path]| *mark == Piece::Gitdir)
            .and_then(|[_mark, path]| path.word())
            .map(|path| Linked(path.to_owned()))
    }

    #[must_use]
    pub fn common(&self) -> Option<Common> {
        self.first().and_then(Piece::lexed::<1>).and_then(|[only]| only.word()).map(|path| Common(path.to_owned()))
    }

    #[must_use]
    pub fn packed(&self, name: &RefName) -> Option<Object> {
        self.0.iter().find_map(|line| {
            Piece::lexed::<3>(line)
                .filter(|[_object, space, _named]| *space == Piece::Space)
                .filter(|[_object, _space, named]| named.word() == Some(name.as_ref()))
                .and_then(|[object, _space, _named]| object.object())
        })
    }

    fn first(&self) -> Option<&str> {
        self.0.first().map(String::as_str)
    }
}

impl AsRef<str> for Object {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for RefName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Linked {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Common {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
