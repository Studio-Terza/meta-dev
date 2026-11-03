use crate::characters::Characters;
use crate::document::key::Key;
use crate::document::text::Text;
use crate::excerpt::Excerpt;
use crate::finding::Finding;
use crate::word::Word;

mod inline;
mod unquoted;
use inline::Inline;
use unquoted::Unquoted;
use unquoted::hidden::Hidden;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Command<'text>(Text<'text>);

impl<'text> Command<'text> {
    const LIMIT: Characters = Characters::new(120);
    const SEPARATORS: [Word; 5] = [Word::new("|"), Word::new("&"), Word::new(";"), Word::new(">"), Word::new("<")];
    const SUBSTITUTIONS: [Word; 2] = [Word::new("`"), Word::new("$(")];
    const OPENERS: [Word; 2] = [Word::new("!"), Word::new("(")];
    const KEYWORDS: [Word; 7] = [
        Word::new("if"),
        Word::new("for"),
        Word::new("while"),
        Word::new("until"),
        Word::new("case"),
        Word::new("["),
        Word::new("[["),
    ];
    const SHELLS: [Word; 5] =
        [Word::new("bash"), Word::new("sh"), Word::new("pwsh"), Word::new("powershell"), Word::new("cmd")];

    pub(crate) fn new(text: Text<'text>) -> Command<'text> {
        Command(text.trimmed())
    }

    pub(crate) fn finding(self, setting: Key) -> Option<Finding> {
        self.0
            .multiline()
            .map(|_| Finding::Block)
            .or_else(|| self.excess().map(|characters| Finding::TooLong { characters, limit: Self::LIMIT }))
            .or_else(|| Self::operator_in(self, setting).map(|symbol| Finding::Operator { symbol }))
            .or_else(|| Self::keyword_in(self.0).map(|word| Finding::Control { word }))
            .or_else(|| {
                (setting == Key::Run)
                    .then(|| Unquoted::new(self.0, Hidden::Quoted).parsed::<Inline>().ok())
                    .flatten()
                    .and_then(Inline::program)
                    .map(|word| Finding::Inline { word })
            })
            .or_else(|| {
                (setting == Key::Shell)
                    .then(|| self.foreign_shell())
                    .flatten()
                    .map(|word| Finding::Interpreter { name: Excerpt::from(word) })
            })
    }

    fn excess(self) -> Option<Characters> {
        Some(self.0.count()).filter(|characters| *characters > Self::LIMIT)
    }

    fn foreign_shell(self) -> Option<Text<'text>> {
        self.0.first_word_outside(&Self::SHELLS)
    }

    fn operator_in(command: Command<'_>, setting: Key) -> Option<Word> {
        let separators = if setting == Key::Shell {
            Unquoted::new(command.0, Hidden::Nothing)
        } else {
            Unquoted::new(command.0, Hidden::Quoted)
        };
        separators
            .first_of(&Self::SEPARATORS)
            .or_else(|| Unquoted::new(command.0, Hidden::Singles).first_of(&Self::SUBSTITUTIONS))
            .or_else(|| command.0.modified_expansion())
    }

    fn keyword_in(text: Text<'_>) -> Option<Word> {
        Self::OPENERS
            .iter()
            .copied()
            .find(|word| text.after(Text::from(*word)).is_some())
            .or_else(|| Self::KEYWORDS.iter().copied().find(|word| text.after_word(*word).is_some()))
    }
}
