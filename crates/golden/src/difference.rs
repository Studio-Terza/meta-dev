mod first_change;
mod line;
mod mark;

use std::fmt;

use first_change::FirstChange;
use line::Line;
use mark::Mark;
use newtype::Newtype;
use text_lines::KeptLines;

use crate::Output;

#[derive(Debug, PartialEq, Eq)]
pub struct Difference {
    before: Vec<Line>,
    recorded: Vec<Line>,
    actual: Vec<Line>,
    after: Vec<Line>,
    first_change: FirstChange,
}

impl Difference {
    const CONTEXT_LEN: usize = 3;

    #[must_use]
    pub fn between(recorded: Output<'_>, actual: Output<'_>) -> Option<Self> {
        (recorded != actual).then(|| Self::of_lines(&Self::lines(recorded), &Self::lines(actual)))
    }

    fn lines(text: Output<'_>) -> Vec<Line> {
        text.into_inner().parse::<KeptLines>().into_iter().flatten().map(Line::new).collect()
    }

    fn of_lines(recorded: &[Line], actual: &[Line]) -> Self {
        let same = |pair: &(&Line, &Line)| pair.0 == pair.1;
        let head = recorded.iter().zip(actual).take_while(same).count();
        let tail = recorded.iter().skip(head).rev().zip(actual.iter().skip(head).rev()).take_while(same).count();
        let middle = |lines: &[Line]| lines.iter().skip(head).take(lines.len() - head - tail).cloned().collect();
        Self {
            before: recorded.iter().take(head).skip(head.saturating_sub(Self::CONTEXT_LEN)).cloned().collect(),
            recorded: middle(recorded),
            actual: middle(actual),
            after: recorded.iter().rev().take(tail).rev().take(Self::CONTEXT_LEN).cloned().collect(),
            first_change: FirstChange::new(head),
        }
    }

    fn marked(formatter: &mut fmt::Formatter<'_>, mark: Mark, lines: &[Line]) -> fmt::Result {
        lines.iter().try_for_each(|line| writeln!(formatter, "{mark} {:?}", line.as_ref()))
    }
}

impl fmt::Display for Difference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "@ first change at line {}; - golden, + actual", self.first_change)?;
        Self::marked(formatter, Mark::Context, &self.before)?;
        Self::marked(formatter, Mark::Recorded, &self.recorded)?;
        Self::marked(formatter, Mark::Actual, &self.actual)?;
        Self::marked(formatter, Mark::Context, &self.after)
    }
}
