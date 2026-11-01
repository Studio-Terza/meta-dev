mod text_lines;

use std::fmt;

use text_lines::TextLines;

#[derive(Debug, PartialEq, Eq)]
pub struct Difference {
    before: Vec<String>,
    recorded: Vec<String>,
    actual: Vec<String>,
    after: Vec<String>,
    first_changed_index: usize,
}

impl Difference {
    const CONTEXT_LEN: usize = 3;

    #[must_use]
    pub fn between(recorded: &str, actual: &str) -> Option<Self> {
        (recorded != actual).then(|| {
            let lines = |text| TextLines::new(text).collect::<Vec<_>>();
            Self::of_lines(&lines(recorded), &lines(actual))
        })
    }

    fn of_lines(recorded: &[&str], actual: &[&str]) -> Self {
        let same = |pair: &(&&str, &&str)| pair.0 == pair.1;
        let head = recorded.iter().zip(actual).take_while(same).count();
        let tail = recorded.iter().skip(head).rev().zip(actual.iter().skip(head).rev()).take_while(same).count();
        let middle = |lines: &[&str]| Self::owned(lines.iter().skip(head).take(lines.len() - head - tail));
        Self {
            before: Self::owned(recorded.iter().take(head).skip(head.saturating_sub(Self::CONTEXT_LEN))),
            recorded: middle(recorded),
            actual: middle(actual),
            after: Self::owned(recorded.iter().rev().take(tail).rev().take(Self::CONTEXT_LEN)),
            first_changed_index: head,
        }
    }

    fn owned<'line>(lines: impl Iterator<Item = &'line &'line str>) -> Vec<String> {
        lines.map(|line| (*line).to_owned()).collect()
    }

    fn marked(formatter: &mut fmt::Formatter<'_>, mark: char, lines: &[String]) -> fmt::Result {
        lines.iter().try_for_each(|line| writeln!(formatter, "{mark} {line:?}"))
    }
}

impl fmt::Display for Difference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "@ first change at line {}; - golden, + actual", self.first_changed_index + 1)?;
        Self::marked(formatter, ' ', &self.before)?;
        Self::marked(formatter, '-', &self.recorded)?;
        Self::marked(formatter, '+', &self.actual)?;
        Self::marked(formatter, ' ', &self.after)
    }
}
