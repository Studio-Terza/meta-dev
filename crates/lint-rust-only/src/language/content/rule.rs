use newtype::Newtype;

use super::Language;
use super::text::Text;
use super::weight::Weight;

#[derive(Clone, Copy)]
pub(super) struct Rule {
    language: Language,
    weight: Weight,
    start: Text<'static>,
    middle: Text<'static>,
    end: Text<'static>,
}

impl Rule {
    const CLEAR: Weight = Weight::new(2);
    const HINT: Weight = Weight::new(1);
    pub(super) const ENOUGH: Weight = Weight::new(2);

    pub(super) const ALL: [Self; 17] = [
        Self {
            language: Language::Swift,
            weight: Self::CLEAR,
            start: Text::new("import XCTest"),
            middle: Text::new(""),
            end: Text::new(""),
        },
        Self {
            language: Language::Swift,
            weight: Self::CLEAR,
            start: Text::new("import SwiftUI"),
            middle: Text::new(""),
            end: Text::new(""),
        },
        Self {
            language: Language::Swift,
            weight: Self::CLEAR,
            start: Text::new("import UIKit"),
            middle: Text::new(""),
            end: Text::new(""),
        },
        Self {
            language: Language::Swift,
            weight: Self::CLEAR,
            start: Text::new("import Foundation"),
            middle: Text::new(""),
            end: Text::new(""),
        },
        Self {
            language: Language::Swift,
            weight: Self::CLEAR,
            start: Text::new("import Cocoa"),
            middle: Text::new(""),
            end: Text::new(""),
        },
        Self {
            language: Language::Swift,
            weight: Self::CLEAR,
            start: Text::new("import AppKit"),
            middle: Text::new(""),
            end: Text::new(""),
        },
        Self {
            language: Language::Swift,
            weight: Self::CLEAR,
            start: Text::new("class "),
            middle: Text::new(": XCTestCase"),
            end: Text::new("{"),
        },
        Self {
            language: Language::Swift,
            weight: Self::CLEAR,
            start: Text::new("final class "),
            middle: Text::new(": XCTestCase"),
            end: Text::new("{"),
        },
        Self {
            language: Language::Python,
            weight: Self::CLEAR,
            start: Text::new("def "),
            middle: Text::new("("),
            end: Text::new(":"),
        },
        Self {
            language: Language::Python,
            weight: Self::CLEAR,
            start: Text::new("async def "),
            middle: Text::new("("),
            end: Text::new(":"),
        },
        Self {
            language: Language::Python,
            weight: Self::HINT,
            start: Text::new("from "),
            middle: Text::new(" import "),
            end: Text::new(""),
        },
        Self {
            language: Language::Python,
            weight: Self::HINT,
            start: Text::new("import "),
            middle: Text::new(""),
            end: Text::new(""),
        },
        Self {
            language: Language::Shell,
            weight: Self::CLEAR,
            start: Text::new("set -"),
            middle: Text::new(""),
            end: Text::new(""),
        },
        Self {
            language: Language::Shell,
            weight: Self::CLEAR,
            start: Text::new("if "),
            middle: Text::new(""),
            end: Text::new("; then"),
        },
        Self {
            language: Language::Shell,
            weight: Self::CLEAR,
            start: Text::new("for "),
            middle: Text::new(""),
            end: Text::new("; do"),
        },
        Self {
            language: Language::C,
            weight: Self::CLEAR,
            start: Text::new("#include <"),
            middle: Text::new(""),
            end: Text::new(">"),
        },
        Self {
            language: Language::C,
            weight: Self::CLEAR,
            start: Text::new("#include \""),
            middle: Text::new(""),
            end: Text::new("\""),
        },
    ];

    pub(super) fn language(&self) -> Language {
        self.language
    }

    pub(super) fn weighs(&self, line: Text<'_>) -> Option<Weight> {
        let start = self.start.into_inner();
        let middle = self.middle.into_inner();
        let end = self.end.into_inner();
        Some(line.into_inner())
            .filter(|line| line.get(..start.len()) == Some(start))
            .filter(|line| line.len().checked_sub(end.len()).and_then(|from| line.get(from..)) == Some(end))
            .filter(|line| line.contains(middle))
            .map(|_| self.weight)
    }
}
