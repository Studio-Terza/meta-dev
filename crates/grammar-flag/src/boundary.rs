use grammar::logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
#[logos(crate = grammar::logos)]
enum Spelling {
    #[token("--")]
    Lead,
}

#[derive(Debug)]
pub struct Flags;

impl Flags {
    #[must_use]
    pub fn spelled(text: &str) -> Option<&str> {
        Spelling::lexer(text).next().and_then(Result::ok).map(|_lead| text)
    }
}
