use std::collections::BTreeMap;
use std::io::Write;

use console_ports::Heard;
use design_runs::{Account, FailuresReported, Unreported, UnreportedFailures};
use design_store::Recalled;
use grammar_search::Terms;
use newtype::Wrapping;
use serde::{Deserialize, Serialize};
use store_ports::Store;
use transcript_ports::Body;

use crate::cli::failure::Failure;
use crate::cli::output::Count;
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

#[derive(Debug, Deserialize)]
pub struct Prompting {
    prompt: Option<Body>,
}

#[derive(Debug, Deserialize)]
pub struct Stopping {
    #[serde(rename = "stop_hook_active")]
    continuation: Continuation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Continuing;

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(from = "bool")]
struct Continuation(Option<Continuing>);

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Decision {
    Block {
        decision: Verdict,
        reason: String,
    },
    Notice {
        #[serde(rename = "systemMessage")]
        system_message: String,
    },
    Context {
        #[serde(rename = "hookSpecificOutput")]
        specific: Specific,
    },
}

#[derive(Debug, Serialize)]
pub struct Specific {
    #[serde(rename = "hookEventName")]
    event: HookEvent,
    #[serde(rename = "additionalContext")]
    context: String,
}

#[derive(Debug, Serialize)]
pub enum Verdict {
    #[serde(rename = "block")]
    Block,
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
        serde_json::to_writer_pretty(&mut *out, self).map_err(Failure::Render)?;
        writeln!(out)?;
        Ok(Outcome::Settings)
    }
}

impl Stopping {
    pub fn heard(heard: Heard) -> Result<Stopping, Failure> {
        serde_json::from_str(&heard.into_inner()).map_err(Failure::Unheard)
    }

    pub fn decided(
        &self,
        concluded: Result<Count, Failure>,
        store: &impl Store,
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        match self.continuation.0 {
            Some(Continuing) => concluded.map_or_else(
                |halt| Decision::Notice { system_message: halt.to_string() }.written(out),
                |_jobs| Ok(Outcome::Hooked(HookEvent::Stop)),
            ),
            None => Decision::told(store, HookEvent::Stop, &[], out),
        }
    }
}

impl From<bool> for Continuation {
    fn from(active: bool) -> Continuation {
        Continuation(active.then_some(Continuing))
    }
}

impl Decision {
    const RECALLED: &'static str = "Design nodes matching this message (design show <id>):";

    pub fn told(
        store: &impl Store,
        event: HookEvent,
        recalled: &[Recalled],
        out: &mut impl Write,
    ) -> Result<Outcome, Failure> {
        let unreported = store.read(UnreportedFailures)?;
        let accounts = unreported.iter().map(|failure| AsRef::<Account>::as_ref(failure).to_string());
        let nodes = recalled
            .first()
            .map(|_first| Self::RECALLED.to_owned())
            .into_iter()
            .chain(recalled.iter().map(Recalled::to_string));
        let told: Vec<String> = accounts.chain(nodes).collect();
        told.first().map(|_first| told.join("\n")).map_or(Ok(Outcome::Hooked(event)), |joined| {
            let outcome = Decision::telling(event, joined).written(out)?;
            store.write(unreported.iter().map(Unreported::job).collect::<FailuresReported>())?;
            Ok(outcome)
        })
    }

    fn telling(event: HookEvent, told: String) -> Decision {
        match event {
            HookEvent::Stop => Decision::Block { decision: Verdict::Block, reason: told },
            HookEvent::UserPromptSubmit => Decision::Context { specific: Specific { event, context: told } },
        }
    }

    fn written(&self, out: &mut impl Write) -> Result<Outcome, Failure> {
        serde_json::to_writer(&mut *out, self).map_err(Failure::Render)?;
        writeln!(out)?;
        Ok(self.outcome())
    }

    fn outcome(&self) -> Outcome {
        match self {
            Decision::Block { .. } => Outcome::Blocked,
            Decision::Notice { .. } => Outcome::Noticed,
            Decision::Context { .. } => Outcome::Told,
        }
    }
}

impl Prompting {
    pub fn heard(heard: Heard) -> Result<Prompting, Failure> {
        let text = heard.into_inner();
        text.is_empty()
            .then_some(Prompting { prompt: None })
            .map_or_else(|| serde_json::from_str(&text).map_err(Failure::Unheard), Ok)
    }

    pub fn terms(&self) -> Option<Terms> {
        self.prompt.as_ref().map(Body::to_string).and_then(|prompt| Terms::lexed([prompt.as_str()]))
    }
}
