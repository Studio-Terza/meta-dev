mod lexeme;

use proc_macro2::{Delimiter, Ident, TokenStream};
use syn::Macro;
use syn::parse::ParseStream;

use code_model::{Found, Implementation, Origin, TraitPath};
use lint_layout::{LineNumber, Name};

use super::scopes::Scopes;
use lexeme::{Depth, Lexeme, Mark};

#[derive(Debug)]
pub(super) struct MacroImpls;

impl MacroImpls {
    const MACRO_RULES: &'static str = "macro_rules";
    const IMPL: &'static str = "impl";
    const FOR: &'static str = "for";
    const METAVARIABLE: char = '$';
    const OPEN: char = '<';
    const CLOSE: char = '>';
    const DASH: char = '-';

    pub(super) fn in_macro(found: &Macro, scopes: &Scopes) -> Vec<Found<Implementation>> {
        Some(found)
            .filter(|found| found.path.is_ident(Self::MACRO_RULES))
            .and_then(|found| found.parse_body_with(|input: ParseStream<'_>| input.parse::<TokenStream>()).ok())
            .map(|body| Self::written(&Lexeme::sequence(body), scopes))
            .unwrap_or_default()
    }

    fn written(lexemes: &[Lexeme], scopes: &Scopes) -> Vec<Found<Implementation>> {
        lexemes
            .iter()
            .enumerate()
            .flat_map(|(at, lexeme)| lexeme.found(lexemes.get(at + 1..).unwrap_or_default(), scopes))
            .collect()
    }

    fn header(rest: &[Lexeme], keyword: &Ident, scopes: &Scopes) -> Option<Found<Implementation>> {
        let end = rest.iter().position(|lexeme| lexeme.delimiter() == Some(Delimiter::Brace)).unwrap_or(rest.len());
        let header = rest.get(..end)?;
        let at = header
            .iter()
            .zip(Lexeme::depths(header))
            .enumerate()
            .filter(|(_, (_, depth))| *depth == Depth::Outer)
            .find(|(_, (lexeme, _))| lexeme.word().is_some_and(|ident| *ident == Self::FOR))
            .map(|(at, _)| at)?;
        let subject = Self::subject(header.get(at + 1..)?);
        let implementation =
            Implementation::new(subject, Some(scopes.scoped(&Self::path(header.get(..at)?))), Origin::Written);
        Some(Found::new(LineNumber::from(keyword.span()), implementation))
    }

    fn subject(after: &[Lexeme]) -> Name {
        after
            .iter()
            .find(|lexeme| lexeme.mark() != Mark::Metavariable)
            .and_then(Lexeme::word)
            .map(Name::from)
            .unwrap_or_default()
    }

    fn path(lexemes: &[Lexeme]) -> TraitPath {
        lexemes
            .iter()
            .zip(Lexeme::depths(lexemes))
            .enumerate()
            .filter(|(_, (_, depth))| *depth == Depth::Outer)
            .filter(|(at, _)| {
                at.checked_sub(1).and_then(|previous| lexemes.get(previous)).map(Lexeme::mark)
                    != Some(Mark::Metavariable)
            })
            .filter_map(|(_, (lexeme, _))| lexeme.word().map(Name::from))
            .collect()
    }
}
