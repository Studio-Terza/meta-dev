use std::path::Path;
mod counted_name;
mod sqlx_name;

use std::borrow::Borrow;

use syntax_lexer::TokenTable;

use lint_core::{Finding, Site, SourceFile, Violation};
use lint_file::{Leader, Led};
use lint_layout::{LineNumber, Name, Whole, Word};
use lint_sweep::Member;

pub(crate) use counted_name::CountedName;
pub(crate) use sqlx_name::SqlxName;

pub(crate) trait Reserved: Finding + Default {
    const OWNERS: [Word<'static>; 2];
    const NAMES: &'static [Word<'static>];
}

#[derive(Debug)]
struct Spelled {
    name: Name,
    line: LineNumber,
}

#[derive(Debug)]
pub struct ReservedNames;

impl Leader for ReservedNames {
    fn led(file: &SourceFile) -> Led<'_> {
        Led::from(Self::sorted(file))
    }
}

impl ReservedNames {
    pub(crate) const SQLX: Member = Member::new(1);

    fn sorted(file: &SourceFile) -> Vec<Vec<Violation<'_>>> {
        let text = file.text();
        let mentioned = text.mentioning(CountedName::NAMES).or_else(|| text.mentioning(SqlxName::NAMES));
        let spelled = mentioned.map_or_default(|_| Self::idents(file.tokens()));
        vec![Self::kept::<CountedName>(file, &spelled), Self::kept::<SqlxName>(file, &spelled)]
    }

    fn idents(table: &TokenTable) -> Vec<Spelled> {
        let words = table.lexemes().iter().filter_map(|lexeme| {
            let named = lexeme.kind().named()?;
            Some(Spelled {
                name: Name::from(table.spelled(*named.as_ref())),
                line: LineNumber::from(lexeme.place().line()),
            })
        });
        words.collect()
    }

    fn kept<'file, Reserving: Reserved>(file: &'file SourceFile, spelled: &[Spelled]) -> Vec<Violation<'file>> {
        let path = AsRef::<Path>::as_ref(file.head());
        let name: &Name = file.head().placement().package().crate_name().borrow();
        let outside = name.matches::<Whole>(&Reserving::OWNERS).map_or(Some(spelled), |_| None);
        let named =
            outside.into_iter().flatten().filter(|found| found.name.matches::<Whole>(Reserving::NAMES).is_some());
        named.map(|found| Violation::new(Site::new(path, found.line), Reserving::default())).collect()
    }
}
