use grammar::logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Head {
    #[regex("[a-z]+")]
    Kind,
    #[token(":")]
    Colon,
}

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Tail {
    #[regex("[A-Za-z0-9_][A-Za-z0-9_.:/-]*")]
    Name,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spelled<'text> {
    kind: &'text str,
    name: &'text str,
}

#[derive(Debug)]
pub struct Artifacts;

impl Artifacts {
    #[must_use]
    pub fn spelled(text: &str) -> Option<Spelled<'_>> {
        let mut head = Head::lexer(text);
        let kind = head.next().filter(|lexed| *lexed == Ok(Head::Kind)).map(|_kind| head.slice());
        let colon = head.next().filter(|lexed| *lexed == Ok(Head::Colon));
        let mut tail = head.morph::<Tail>();
        let name = tail.next().filter(|lexed| *lexed == Ok(Tail::Name)).map(|_name| tail.slice());
        let rest: Vec<Result<Tail, ()>> = tail.collect();
        kind.zip(name)
            .zip(colon)
            .filter(|_spelled| rest.is_empty())
            .map(|((spelled_kind, spelled_name), _colon)| Spelled { kind: spelled_kind, name: spelled_name })
    }
}

impl<'text> Spelled<'text> {
    pub fn joined<Joined>(self, join: impl FnOnce(&'text str, &'text str) -> Joined) -> Joined {
        join(self.kind, self.name)
    }
}
