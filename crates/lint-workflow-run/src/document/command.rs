use crate::characters::Characters;
use crate::document::key::Key;
use crate::document::text::Text;
use crate::finding::Finding;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Command<'text>(Text<'text>);

impl<'text> Command<'text> {
    const LIMIT: Characters = Characters::new(120);
    const OPERATORS: [&'static str; 7] = ["|", "&", ";", ">", "<", "`", "$("];
    const OPENERS: [&'static str; 2] = ["!", "("];
    const KEYWORDS: [&'static str; 7] = ["if", "for", "while", "until", "case", "[", "[["];
    const SHELLS: [&'static str; 5] = ["bash", "sh", "pwsh", "powershell", "cmd"];

    pub(crate) fn new(text: &'text str) -> Command<'text> {
        Command(Text::new(text).trimmed())
    }

    pub(crate) fn finding(self, setting: Key) -> Option<Finding> {
        self.0
            .spans_lines()
            .then_some(Finding::Block)
            .or_else(|| self.excess().map(|characters| Finding::TooLong { characters, limit: Self::LIMIT }))
            .or_else(|| Self::operator_in(self).map(|symbol| Finding::Operator { symbol }))
            .or_else(|| Self::keyword_in(self.0).map(|word| Finding::Control { word }))
            .or_else(|| {
                (setting == Key::Shell)
                    .then(|| self.foreign_shell())
                    .flatten()
                    .map(|word| Finding::Interpreter { name: word.to_owned() })
            })
    }

    fn excess(self) -> Option<Characters> {
        Some(self.0.count()).filter(|characters| *characters > Self::LIMIT)
    }

    fn foreign_shell(self) -> Option<&'text str> {
        self.0.first_word_outside(&Self::SHELLS)
    }

    fn operator_in(command: Command<'_>) -> Option<&'static str> {
        command.0.first_inside(&Self::OPERATORS).or_else(|| command.0.modified_expansion().map(|_| Text::EXPANSION))
    }

    fn keyword_in(text: Text<'_>) -> Option<&'static str> {
        Self::OPENERS
            .iter()
            .copied()
            .find(|word| text.after(Text::new(word)).is_some())
            .or_else(|| Self::KEYWORDS.iter().copied().find(|word| text.after_word(word).is_some()))
    }
}
