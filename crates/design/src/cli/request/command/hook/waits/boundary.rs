use std::collections::BTreeMap;
use std::ops::Range;
use std::{fmt, iter};

use dispatch_ports::{Reply, Wall};
use invariants::Ignoring;
use newtype::Wrapping;
use serde::Deserialize;
use serde_json::Value;
use transcript_ports::Body;

use super::super::answers::Replied;
use crate::cli::request::ignored::Ignored;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Shell {
    what: String,
    began: Option<i64>,
}

#[derive(Debug, Default)]
pub struct Snapshot {
    shells: BTreeMap<String, Shell>,
    described: BTreeMap<String, String>,
    heard: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stall {
    Deadlock { on: String },
    Overrun { id: String, what: String, minutes: i64 },
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum On {
    Owner,
    Shell,
    Agent,
    Nothing,
}

#[derive(Debug, Deserialize)]
pub struct Waiting {
    #[serde(rename = "waiting_on")]
    on: On,
    #[serde(default)]
    target: String,
    #[serde(default)]
    deadlock: bool,
}

#[derive(Debug)]
pub struct Asking<'snapshot> {
    snapshot: &'snapshot Snapshot,
    said: &'snapshot Body,
}

struct Stamp;

impl Stamp {
    const FIELDS: [Range<usize>; 6] = [0..4, 5..7, 8..10, 11..13, 14..16, 17..19];

    fn seconds(stamp: &str) -> Option<i64> {
        let numbers: Vec<i64> = Self::FIELDS
            .iter()
            .filter_map(|range| stamp.get(range.clone()))
            .filter_map(|digits| digits.parse::<i64>().into_iter().next())
            .collect();
        let [year, month, day, hour, minute, second] = <[i64; 6]>::try_from(numbers).ignoring(Ignored::Unstamped)?;
        Some(Self::days(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
    }

    fn days(year: i64, month: i64, day: i64) -> i64 {
        let shifted = if month <= 2 { year - 1 } else { year };
        let era = shifted.div_euclid(400);
        let of_era = shifted - era * 400;
        let from_march = (month + 9).rem_euclid(12);
        let of_year = (153 * from_march + 2) / 5 + day - 1;
        let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
        era * 146_097 + of_cycle - 719_468
    }
}

struct Cut;

impl Cut {
    fn between<'text>(text: &'text str, open: &str, close: &str) -> Option<&'text str> {
        let (at, _open) = text.match_indices(open).next()?;
        let rest = text.get(at + open.len()..)?;
        let (end, _close) = rest.match_indices(close).next()?;
        rest.get(..end)
    }

    fn short(text: &str, kept: usize) -> String {
        text.chars().take(kept).map(|character| if character == '\n' { ' ' } else { character }).collect()
    }
}

impl Snapshot {
    const HEARD: usize = 40;
    const SHOWN: usize = 160;
    const LIMIT_MINUTES: i64 = 60;
    const BACKGROUND: &'static str = "running in background with ID: ";
    const TASK: [&'static str; 2] = ["<task-id>", "</task-id>"];
    const STATUS: [&'static str; 2] = ["<status>", "</status>"];
    const TEAMMATE: [&'static str; 2] = ["<teammate-message teammate_id=\"", "\""];

    #[must_use]
    pub fn read(bytes: &[u8]) -> Snapshot {
        let ends: Vec<usize> = bytes
            .iter()
            .enumerate()
            .filter_map(|(at, byte)| (*byte == b'\n').then_some(at))
            .chain([bytes.len()])
            .collect();
        let starts = iter::once(0).chain(ends.iter().map(|end| end + 1));
        let read = starts
            .zip(&ends)
            .filter_map(|(start, end)| bytes.get(start..*end))
            .filter_map(|line| serde_json::from_slice::<Value>(line).into_iter().next())
            .fold(Snapshot::default(), |snapshot, entry| snapshot.entry(&entry));
        let skipped = read.heard.len().saturating_sub(Self::HEARD);
        Snapshot { heard: read.heard.into_iter().skip(skipped).collect(), ..read }
    }

    fn entry(self, entry: &Value) -> Snapshot {
        let at = entry.get("timestamp").and_then(Value::as_str).and_then(Stamp::seconds);
        let items: Vec<&Value> = match entry.pointer("/message/content") {
            Some(Value::Array(items)) => items.iter().collect(),
            Some(text @ Value::String(_)) => vec![text],
            Some(Value::Null | Value::Bool(_) | Value::Number(_) | Value::Object(_)) | None => Vec::new(),
        };
        items.into_iter().fold(self, |snapshot, item| snapshot.item(item, at))
    }

    fn item(self, item: &Value, at: Option<i64>) -> Snapshot {
        match item.get("type").and_then(Value::as_str) {
            Some("tool_use") => self.used(item),
            _ => match Self::text(item) {
                Some(text) => self.heard(&text, item.get("tool_use_id").and_then(Value::as_str), at),
                None => self,
            },
        }
    }

    fn text(item: &Value) -> Option<String> {
        let direct = item.as_str().or_else(|| item.get("text").and_then(Value::as_str));
        let nested = || match item.get("content") {
            Some(Value::String(text)) => Some(text.clone()),
            Some(Value::Array(parts)) => {
                Some(parts.iter().filter_map(|part| part.get("text").and_then(Value::as_str)).collect())
            }
            Some(Value::Null | Value::Bool(_) | Value::Number(_) | Value::Object(_)) | None => None,
        };
        direct.map(str::to_owned).or_else(nested)
    }

    fn used(mut self, item: &Value) -> Snapshot {
        let input = item.get("input");
        let field = |name: &str| input.and_then(|given| given.get(name)).and_then(Value::as_str).unwrap_or_default();
        let backgrounded = input.and_then(|given| given.get("run_in_background")).and_then(Value::as_bool);
        match item.get("name").and_then(Value::as_str) {
            Some("Bash") if backgrounded == Some(true) => {
                let id = item.get("id").and_then(Value::as_str).unwrap_or_default().to_owned();
                let what = format!("{}: {}", field("description"), field("command"));
                self.described.insert(id, Cut::short(&what, Self::SHOWN));
            }
            Some("Agent") => {
                self.heard.push(format!("lead started agent {} ({})", field("name"), field("description")));
            }
            Some("SendMessage") => {
                self.heard.push(format!("lead sent to {}: {}", field("to"), Cut::short(field("message"), Self::SHOWN)));
            }
            _ => {}
        }
        self
    }

    fn heard(mut self, text: &str, tool: Option<&str>, at: Option<i64>) -> Snapshot {
        let launched = Cut::between(text, Self::BACKGROUND, ".");
        if let Some(id) = launched {
            let what = tool.and_then(|used| self.described.get(used)).cloned().unwrap_or_default();
            self.shells.insert(id.to_owned(), Shell { what, began: at });
        }
        let [task_open, task_close] = Self::TASK;
        let [status_open, status_close] = Self::STATUS;
        if let Some(task) = Cut::between(text, task_open, task_close) {
            self.shells.remove(task);
            let status = Cut::between(text, status_open, status_close).unwrap_or_default();
            self.heard.push(format!("task {task} {status}"));
        }
        let [teammate_open, teammate_close] = Self::TEAMMATE;
        if let Some(name) = Cut::between(text, teammate_open, teammate_close) {
            self.heard.push(format!("agent {name} said: {}", Cut::short(text, Self::SHOWN)));
        }
        self
    }

    #[must_use]
    pub fn overrun(&self, now: Wall) -> Option<Stall> {
        let seconds = now.into_inner();
        self.shells.iter().find_map(|(id, shell)| {
            let minutes = shell.began.map(|began| (seconds - began) / 60)?;
            (minutes > Self::LIMIT_MINUTES).then(|| Stall::Overrun {
                id: id.clone(),
                what: shell.what.clone(),
                minutes,
            })
        })
    }

    #[must_use]
    pub fn asking<'snapshot>(&'snapshot self, said: &'snapshot Body) -> Asking<'snapshot> {
        Asking { snapshot: self, said }
    }
}

impl Waiting {
    pub fn read(reply: &Reply) -> Option<Waiting> {
        Replied::of(reply).object::<Waiting>().ignoring(Ignored::Unclassified)
    }

    pub fn deadlocked(self) -> Option<Stall> {
        let on = match self.on {
            On::Owner => "the owner".to_owned(),
            On::Shell => format!("shell {}", self.target),
            On::Agent => format!("agent {}", self.target),
            On::Nothing => "nothing".to_owned(),
        };
        self.deadlock.then_some(Stall::Deadlock { on })
    }
}

impl fmt::Display for Asking<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "You check why a lead agent ended its turn. Reply with JSON only: \
             {{\"waiting_on\": \"owner\" | \"shell\" | \"agent\" | \"nothing\", \"target\": \"<shell id or agent \
             name, else empty>\", \"deadlock\": true | false}}.\n\
             waiting_on is owner when the message asks the owner something or reports to the owner with no further \
             work promised; shell or agent when it says it will act once that shell or agent finishes; nothing \
             otherwise.\n\
             deadlock is true when the message promises further work (merging, continuing, acting on a result) but \
             nothing below will wake the lead: no background shell is still running, and every agent it waits on \
             has gone idle or sent its final report, or is itself waiting on the lead. A lead that waits on the \
             owner for an answer the owner already gave is also deadlocked. When unsure, answer true.\n"
        )?;
        writeln!(formatter, "Background shells still running:")?;
        if self.snapshot.shells.is_empty() {
            writeln!(formatter, "- none")?;
        }
        self.snapshot.shells.iter().try_for_each(|(id, shell)| writeln!(formatter, "- {id}: {}", shell.what))?;
        writeln!(formatter, "\nRecent agent and task events, oldest first:")?;
        self.snapshot.heard.iter().try_for_each(|heard| writeln!(formatter, "- {heard}"))?;
        write!(formatter, "\nThe lead's final message:\n{}", self.said)
    }
}

impl fmt::Display for Stall {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stall::Deadlock { on } => write!(
                formatter,
                "STALLED: you ended your turn waiting on {on}, and nothing running will wake you (owner 06cf2534). \
                 Do the next step now, or start the shell or agent you mean to wait on."
            ),
            Stall::Overrun { id, what, minutes } => write!(
                formatter,
                "OVERRUN: background shell {id} ({what}) has run {minutes} minutes, past the {}-minute limit, and \
                 may never end (owner 06cf2534). Check its output, then kill or replace it.",
                Snapshot::LIMIT_MINUTES
            ),
        }
    }
}
