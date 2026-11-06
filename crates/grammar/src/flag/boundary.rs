use logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
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
