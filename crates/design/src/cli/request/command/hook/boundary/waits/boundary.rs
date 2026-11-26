use std::fmt;
use std::ops::Not;

use design_model::{Awaited, Halt};
use design_store::HeardEvent;
use dispatch_ports::{Reply, Wall};
use invariants::Ignoring;
use newtype::Wrapping;
use serde::Deserialize;
use transcript_ports::{Body, Happening, Instant};

use crate::cli::request::command::hook::answers::Replied;
use crate::cli::request::ignored::Ignored;

#[derive(Debug, Default)]
pub struct Snapshot {
    events: Vec<HeardEvent>,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum On {
    Owner,
    Shell,
    Agent,
    Nothing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Overrun {
    tool: Body,
    minutes: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stall {
    Deadlock(On),
    Overrun(Overrun),
}

#[derive(Clone, Copy, Debug)]
struct Deadlocked;

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(from = "bool")]
struct Lock(Option<Deadlocked>);

#[derive(Debug, Deserialize)]
pub struct Waiting {
    #[serde(rename = "waiting_on")]
    on: On,
    #[serde(default)]
    deadlock: Lock,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Wait {
    awaited: Awaited,
    stall: Option<Stall>,
}

#[derive(Debug)]
pub struct Asking<'snapshot> {
    snapshot: &'snapshot Snapshot,
    said: &'snapshot Body,
    now: Wall,
}

#[derive(Debug)]
struct Aged<'event> {
    event: &'event HeardEvent,
    minutes: Option<i64>,
}

impl From<Vec<HeardEvent>> for Snapshot {
    fn from(events: Vec<HeardEvent>) -> Snapshot {
        Snapshot { events }
    }
}

impl From<bool> for Lock {
    fn from(deadlocked: bool) -> Lock {
        Lock(deadlocked.then_some(Deadlocked))
    }
}

impl From<On> for Awaited {
    fn from(on: On) -> Awaited {
        match on {
            On::Owner => Awaited::Owner,
            On::Shell => Awaited::Shell,
            On::Agent => Awaited::Agent,
            On::Nothing => Awaited::Nothing,
        }
    }
}

impl From<Waiting> for Wait {
    fn from(waiting: Waiting) -> Wait {
        let stall = waiting.deadlock.0.map(|Deadlocked| Stall::Deadlock(waiting.on));
        Wait { awaited: Awaited::from(waiting.on), stall }
    }
}

impl From<Overrun> for Wait {
    fn from(overrun: Overrun) -> Wait {
        Wait { awaited: Awaited::Unchecked, stall: Some(Stall::Overrun(overrun)) }
    }
}

impl From<Awaited> for Wait {
    fn from(awaited: Awaited) -> Wait {
        Wait { awaited, stall: None }
    }
}

impl Wait {
    #[must_use]
    pub fn awaited(&self) -> Awaited {
        self.awaited
    }

    #[must_use]
    pub fn stall(&self) -> Option<&Stall> {
        self.stall.as_ref()
    }

    #[must_use]
    pub fn halt(&self) -> Halt {
        self.stall.as_ref().map_or(Halt::Live, Halt::from)
    }
}

impl From<&Stall> for Halt {
    fn from(stall: &Stall) -> Halt {
        match stall {
            Stall::Deadlock(_) => Halt::Deadlock,
            Stall::Overrun(_) => Halt::Overrun,
        }
    }
}

impl<'event> Aged<'event> {
    const MINUTE: i64 = 60;

    fn since(event: &'event HeardEvent, now: Wall) -> Aged<'event> {
        let minutes = event.began().map(|began| (now.into_inner() - began.into_inner()) / Self::MINUTE);
        Aged { event, minutes }
    }
}

impl Snapshot {
    const LIMIT_MINUTES: i64 = 60;
    const HEARD: usize = 40;
    const SHOWN: usize = 300;

    fn running(&self, now: Wall) -> Vec<Aged<'_>> {
        let notices: Vec<&str> = self
            .events
            .iter()
            .filter(|event| event.happening() == Happening::Notice)
            .map(|event| AsRef::<Body>::as_ref(event).as_ref())
            .collect();
        self.events
            .iter()
            .filter(|event| event.happening() == Happening::Launch)
            .filter(|launch| {
                let tool: &str = AsRef::<Body>::as_ref(launch).as_ref();
                notices.iter().any(|notice| notice.contains(tool)).not()
            })
            .map(|launch| Aged::since(launch, now))
            .collect()
    }

    fn heard(&self) -> Vec<&HeardEvent> {
        let said: Vec<&HeardEvent> =
            self.events.iter().filter(|event| event.happening() != Happening::Launch).collect();
        let skipped = said.len().saturating_sub(Self::HEARD);
        said.into_iter().skip(skipped).collect()
    }

    #[must_use]
    pub fn overrun(&self, now: Wall) -> Option<Overrun> {
        self.running(now).into_iter().find_map(|aged| {
            let minutes = aged.minutes.filter(|minutes| *minutes > Self::LIMIT_MINUTES)?;
            Some(Overrun { tool: AsRef::<Body>::as_ref(aged.event).clone(), minutes })
        })
    }

    #[must_use]
    pub fn asking<'snapshot>(&'snapshot self, said: &'snapshot Body, now: Wall) -> Asking<'snapshot> {
        Asking { snapshot: self, said, now }
    }
}

impl Waiting {
    pub fn read(reply: &Reply) -> Option<Waiting> {
        Replied::from(reply).object::<Waiting>().ignoring(Ignored::Unclassified)
    }
}

impl fmt::Display for Aged<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text: &str = AsRef::<Body>::as_ref(self.event).as_ref();
        let shown: String = text.chars().take(Snapshot::SHOWN).collect();
        let minutes = self.minutes.map_or_else(String::new, |minutes| format!(" ({minutes} min ago)"));
        let at: &Instant = self.event.as_ref();
        write!(formatter, "- {} {at}{minutes}: {shown}", self.event.happening())
    }
}

impl fmt::Display for Asking<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "You check why a lead agent ended its turn. Reply with JSON only: \
             {{\"waiting_on\": \"owner\" | \"shell\" | \"agent\" | \"nothing\", \"deadlock\": true | false}}.\n\
             waiting_on is owner when the message asks the owner something or reports to the owner with no further \
             work promised; shell or agent when it says it will act once a background task or an agent finishes; \
             nothing otherwise.\n\
             deadlock is true when the message promises further work (merging, continuing, acting on a result) but \
             nothing below will wake the lead: no background task is still running, and every agent it waits on \
             has gone idle or sent its final report, or is itself waiting on the lead. A lead that waits on the \
             owner for an answer the owner already gave is also deadlocked. When unsure, answer true.\n\
             A \"sent\" event is the lead messaging the agent it names; an agent the lead sent to after that \
             agent's last notice is working again and will notify the lead, so waiting on it is not a deadlock. \
             A message that answers the owner and promises no further work waits on the owner, not on nothing.\n"
        )?;
        let running: Vec<String> = self.snapshot.running(self.now).iter().map(ToString::to_string).collect();
        let shown = running.first().map_or_else(|| vec!["- none".to_owned()], |_first| running.clone());
        writeln!(formatter, "Background tasks still running (launch id):\n{}", shown.join("\n"))?;
        let heard: Vec<String> =
            self.snapshot.heard().into_iter().map(|event| Aged::since(event, self.now).to_string()).collect();
        writeln!(formatter, "\nRecent notices, relayed messages and sends, oldest first:\n{}", heard.join("\n"))?;
        write!(formatter, "\nThe lead's final message:\n{}", self.said)
    }
}

impl fmt::Display for On {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            On::Owner => formatter.write_str("the owner"),
            On::Shell => formatter.write_str("a background task"),
            On::Agent => formatter.write_str("an agent"),
            On::Nothing => formatter.write_str("nothing"),
        }
    }
}

impl fmt::Display for Stall {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stall::Deadlock(on) => write!(
                formatter,
                "STALLED: you ended your turn waiting on {on}, and nothing running will wake you (owner 06cf2534). \
                 Do the next step now, or start the task or agent you mean to wait on."
            ),
            Stall::Overrun(Overrun { tool, minutes }) => write!(
                formatter,
                "OVERRUN: background task {tool} has run {minutes} minutes, past the {}-minute limit, and may never \
                 end (owner 06cf2534). Check its output, then kill or replace it.",
                Snapshot::LIMIT_MINUTES
            ),
        }
    }
}
