use std::str::FromStr;

use grammar::directive::Pieces;

use crate::reading::Reader;
use crate::{Directive, Misread};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Directives(Vec<Directive>);

impl FromStr for Directives {
    type Err = Misread;

    fn from_str(text: &str) -> Result<Directives, Misread> {
        Pieces::lines(text)
            .iter()
            .filter_map(|line| line.pieces().first().map(|_first| Reader::over(line).directive()))
            .collect::<Result<Vec<Directive>, Misread>>()
            .map(Directives)
    }
}

impl IntoIterator for Directives {
    type Item = Directive;
    type IntoIter = std::vec::IntoIter<Directive>;

    fn into_iter(self) -> std::vec::IntoIter<Directive> {
        self.0.into_iter()
    }
}
