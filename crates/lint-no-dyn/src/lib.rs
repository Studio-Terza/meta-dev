use std::path::Path;

use lint_core::{Fix, Headline, Message, Problem, Site, SourceFile, Violation};
use lint_file::{FileLint, Reads};
use lint_layout::{LineNumber, Word, words};
use syntax_lexer::TokenTable;

#[derive(Debug)]
pub struct NoDyn;

impl FileLint for NoDyn {
    const HEADLINE: Headline = Headline::new(
        "no dyn: every type is known at compile time, so code takes a generic, returns impl Trait or names an enum instead of a trait object",
    );
    const READS: Reads = Reads::Every;

    fn violations(file: &SourceFile) -> Vec<Violation<'_>> {
        let path = AsRef::<Path>::as_ref(file.head());
        let mentioned = file.text().mentioning(&Self::KEYWORD).map(|_| file.tokens());
        let lines = mentioned.map_or_default(Self::keyword_lines);
        lines.into_iter().map(|line| Violation::new(Site::new(path, line), Self::TRAIT_OBJECT)).collect()
    }
}

impl NoDyn {
    const KEYWORD: [Word<'static>; 1] = words!["dyn"];

    const TRAIT_OBJECT: Message = Message::new(
        Problem::new("dyn trait object"),
        Fix::new("take a generic, return impl Trait, or list the known types in an enum"),
    );

    fn keyword_lines(table: &TokenTable) -> Vec<LineNumber> {
        let words = table.lexemes().iter().filter_map(|lexeme| {
            let named = lexeme.kind().named()?;
            let spelled = Word::new(table.spelled(*named.as_ref()));
            Self::KEYWORD.contains(&spelled).then(|| LineNumber::from(lexeme.place().line()))
        });
        words.collect()
    }
}
