use super::Language;
use super::weight::Weight;

#[derive(Clone, Copy)]
pub(super) struct Rule {
    language: Language,
    weight: Weight,
    start: &'static str,
    middle: &'static str,
    end: &'static str,
}

impl Rule {
    const CLEAR: Weight = Weight::new(2);
    const HINT: Weight = Weight::new(1);
    pub(super) const ENOUGH: Weight = Weight::new(2);

    pub(super) const ALL: [Rule; 17] = [
        Rule { language: Language::Swift, weight: Self::CLEAR, start: "import XCTest", middle: "", end: "" },
        Rule { language: Language::Swift, weight: Self::CLEAR, start: "import SwiftUI", middle: "", end: "" },
        Rule { language: Language::Swift, weight: Self::CLEAR, start: "import UIKit", middle: "", end: "" },
        Rule { language: Language::Swift, weight: Self::CLEAR, start: "import Foundation", middle: "", end: "" },
        Rule { language: Language::Swift, weight: Self::CLEAR, start: "import Cocoa", middle: "", end: "" },
        Rule { language: Language::Swift, weight: Self::CLEAR, start: "import AppKit", middle: "", end: "" },
        Rule { language: Language::Swift, weight: Self::CLEAR, start: "class ", middle: ": XCTestCase", end: "{" },
        Rule {
            language: Language::Swift,
            weight: Self::CLEAR,
            start: "final class ",
            middle: ": XCTestCase",
            end: "{",
        },
        Rule { language: Language::Python, weight: Self::CLEAR, start: "def ", middle: "(", end: ":" },
        Rule { language: Language::Python, weight: Self::CLEAR, start: "async def ", middle: "(", end: ":" },
        Rule { language: Language::Python, weight: Self::HINT, start: "from ", middle: " import ", end: "" },
        Rule { language: Language::Python, weight: Self::HINT, start: "import ", middle: "", end: "" },
        Rule { language: Language::Shell, weight: Self::CLEAR, start: "set -", middle: "", end: "" },
        Rule { language: Language::Shell, weight: Self::CLEAR, start: "if ", middle: "", end: "; then" },
        Rule { language: Language::Shell, weight: Self::CLEAR, start: "for ", middle: "", end: "; do" },
        Rule { language: Language::C, weight: Self::CLEAR, start: "#include <", middle: "", end: ">" },
        Rule { language: Language::C, weight: Self::CLEAR, start: "#include \"", middle: "", end: "\"" },
    ];

    pub(super) fn language(&self) -> Language {
        self.language
    }

    pub(super) fn weighs(&self, line: &str) -> Option<Weight> {
        Some(line)
            .filter(|line| line.get(..self.start.len()) == Some(self.start))
            .filter(|line| line.len().checked_sub(self.end.len()).and_then(|from| line.get(from..)) == Some(self.end))
            .filter(|line| line.contains(self.middle))
            .map(|_| self.weight)
    }
}
