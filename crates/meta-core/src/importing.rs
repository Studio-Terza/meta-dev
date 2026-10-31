use crate::event::{Event, Record};
use crate::ids::EscapeId;
use crate::line_hash::LineHash;
use crate::side_line::SideLine;
use std::collections::BTreeSet;

/// What importing the side file into the log decides, from the side file's lines and the records
/// already in the log. A line whose hash an earlier `Escaped` record names was imported before a
/// crash cut the side file's rewrite short: it is dropped, never imported twice.
#[derive(Debug)]
pub struct Importing {
    events: Vec<Event>,
    kept: Vec<SideLine>,
    text: String,
}

impl Importing {
    pub fn of(lines: Vec<SideLine>, earlier: &[Record]) -> Self {
        let escaped = earlier.iter().filter(|record| matches!(record.event, Event::Escaped { .. })).count();
        let done: BTreeSet<LineHash> = earlier
            .iter()
            .filter_map(|record| match &record.event {
                Event::Escaped { imported: Some(imported), .. } => Some(imported.line),
                _ => None,
            })
            .collect();
        let mut importing = Self { events: Vec::new(), kept: Vec::new(), text: String::new() };
        let mut already = 0;
        for line in lines {
            match line.escape() {
                None => importing.kept.push(line),
                Some(_) if done.contains(&line.hash()) => already += 1,
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
