use crate::event::{Event, Record};
use crate::ids::EscapeId;
use crate::line_hash::LineHash;
use crate::side_line::SideLine;
use std::collections::BTreeMap;

/// What importing the side file into the log decides, from the side file's lines and the records
/// already in the log. Copies are counted, not just looked for: the log holds as many `Escaped`
/// records naming a line's hash as were imported before a crash cut the side file's rewrite short,
/// so that many copies of the line are dropped and only the surplus is imported. A retry imports
/// nothing twice, and a repeat the crash did not cover is still imported.
#[derive(Debug)]
pub struct Importing {
    events: Vec<Event>,
    kept: Vec<SideLine>,
    text: String,
}

impl Importing {
    pub fn of(lines: Vec<SideLine>, earlier: &[Record]) -> Self {
        let escaped = earlier.iter().filter(|record| matches!(record.event, Event::Escaped { .. })).count();
        let mut done: BTreeMap<LineHash, usize> = BTreeMap::new();
        earlier.iter().for_each(|record| {
            if let Event::Escaped { imported: Some(imported), .. } = &record.event {
                *done.entry(imported.line).or_default() += 1;
            }
        });
        let mut importing = Self { events: Vec::new(), kept: Vec::new(), text: String::new() };
        let mut already = 0;
        for line in lines {
            match line.escape() {
                None => importing.kept.push(line),
                Some(_) if done.get_mut(&line.hash()).is_some_and(take_one) => already += 1,
                Some(escape) => {
                    let id = EscapeId::after(escaped + importing.events.len());
                    let (event, said) = escape.imported(line.hash());
                    importing.text.push_str(&format!("{id} imported: {said}\n"));
                    importing.events.push(event);
                }
            }
        }
        let kept = importing.kept.len();
        let already = match already {
            0 => String::new(),
            already => format!("{already} already in the log from an earlier import, dropped from the side file\n"),
        };
        let unreadable = match kept {
            0 => String::new(),
            kept => format!("{kept} unreadable lines kept in the side file\n"),
        };
        importing.text = match importing.text.is_empty() && already.is_empty() && unreadable.is_empty() {
            true => "the side file holds no escapes to import\n".to_owned(),
            false => format!("{}{already}{unreadable}", importing.text),
        };
        importing
    }

    /// The `Escaped` events to append, the lines the side file keeps, and what to say.
    pub fn into_parts(self) -> (Vec<Event>, Vec<SideLine>, String) {
        (self.events, self.kept, self.text)
    }
}

/// Takes one copy off a count of imported copies, and says whether one was left to take.
fn take_one(left: &mut usize) -> bool {
    let had = *left > 0;
    *left = left.saturating_sub(1);
    had
}
