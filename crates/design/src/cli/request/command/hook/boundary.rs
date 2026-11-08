use std::collections::BTreeMap;
use std::io::Write;

use serde::Serialize;

use crate::cli::failure::Failure;
use crate::{HookEvent, Outcome};

#[derive(Debug, Serialize)]
pub struct Settings {
    hooks: BTreeMap<HookEvent, [Matcher; 1]>,
}

#[derive(Debug, Serialize)]
struct Matcher {
    hooks: [Handler; 1],
}

#[derive(Debug, Serialize)]
struct Handler {
    #[serde(rename = "type")]
    kind: &'static str,
    command: String,
    timeout: u32,
}

#[derive(Debug, Serialize)]
#[serde(tag = "decision")]
pub enum Decision {
    #[serde(rename = "block")]
    Block { reason: String },
}

impl Settings {
    const KIND: &'static str = "command";
    const SECONDS: u32 = 3600;

    pub fn listed(events: &[HookEvent]) -> Settings {
        Settings {
            hooks: events.iter().map(|event| (*event, [Matcher { hooks: [Settings::handler(*event)] }])).collect(),
        }
    }

    fn handler(event: HookEvent) -> Handler {
        Handler {
            kind: Settings::KIND,
            command: format!("design --workspace \"$CLAUDE_PROJECT_DIR\" hook {event}"),
            timeout: Settings::SECONDS,
        }
    }

    pub fn written(&self, out: &mut impl Write) -> Result<Outcome, Failure> {
        serde_json::to_writer_pretty(&mut *out, self)?;
        writeln!(out)?;
        Ok(Outcome::Settings)
    }
}

impl Decision {
    pub fn blocking(halt: &Failure) -> Decision {
        Decision::Block { reason: halt.to_string() }
    }

    pub fn written(&self, out: &mut impl Write) -> Result<Outcome, Failure> {
        serde_json::to_writer(&mut *out, self)?;
        writeln!(out)?;
        Ok(Outcome::Blocked)
    }
}
