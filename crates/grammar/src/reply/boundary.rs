use logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Spelling {
    #[regex(r"[\p{White_Space}]*[^\p{White_Space}\x00][^\x00]*", allow_greedy = true)]
    Inked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inked<'text>(&'text str);

impl<'text> Inked<'text> {
    #[must_use]
    pub fn read(text: &'text str) -> Option<Inked<'text>> {
        let spelled: Vec<Result<Spelling, ()>> = Spelling::lexer(text).collect();
        <[Result<Spelling, ()>; 1]>::try_from(spelled).ok().and_then(|[only]| only.ok()).map(|_inked| Inked(text))
    }
}

impl AsRef<str> for Inked<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}
