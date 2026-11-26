use std::collections::BTreeSet;
use std::ops::Not;
use std::{fmt, slice};

use algebra::{Monoid, Semigroup};
use design_store::{Quote, Said, TurnEnd, TurnEnds, Unabridged};
use dispatch_ports::{Finished, Reply, Stall, Wall, Watched, Watching};
use newtype::Wrapping;
use transcript_ports::{Body, Instant, SessionId, Uuid};

use super::super::hook::answers::{Answers, Asked, Replied};
use crate::cli::failure::Failure;
use crate::cli::request::Token;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Eval {
    session: SessionId,
    from: Instant,
    until: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Label {
    Asks,
    Silent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Labels {
    haiku: Label,
    sonnet: Label,
}

#[derive(Clone, Debug)]
pub struct Called {
    finished: Option<Finished>,
    seconds: i64,
}

#[derive(Clone, Copy, Debug)]
struct Spanned {
    seconds: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Spent {
    tokens: u64,
    seconds: i64,
}

#[derive(Debug)]
pub struct Truth {
    uuids: BTreeSet<Uuid>,
    spent: Spent,
}

#[derive(Debug)]
pub struct Arm {
    quotes: Vec<Quote>,
    spent: Spent,
}

#[derive(Debug)]
pub struct Checked {
    arm: Arm,
    verified: BTreeSet<Uuid>,
}

#[derive(Debug)]
pub struct Judged {
    truth: Truth,
    twelve: BTreeSet<Uuid>,
    candidates: BTreeSet<Uuid>,
    store: Checked,
    base: Checked,
}

#[derive(Debug)]
pub struct Turn {
    asked: Uuid,
    at: Instant,
    labels: Labels,
    judged: Option<Judged>,
}

#[derive(Debug)]
pub struct Evaluated {
    turns: Vec<Turn>,
    tally: Tally,
}

#[derive(Debug)]
pub struct Grounding<'turn> {
    question: &'turn Body,
    reply: Unabridged<'turn>,
    history: &'turn [Said],
}

#[derive(Debug)]
pub struct Verifying<'turn> {
    question: &'turn Body,
    quotes: &'turn [Quote],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Agreement {
    both: usize,
    haiku: usize,
    sonnet: usize,
    neither: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Retrieval {
    judged: usize,
    answered: usize,
    truths: usize,
    twelve_any: usize,
    all_any: usize,
    twelve_truths: usize,
    all_truths: usize,
    truth_tokens: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Picking {
    found_any: usize,
    found: usize,
    reachable: usize,
    picks: usize,
    verified: usize,
    tokens: u64,
    seconds: i64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Tally {
    agreement: Agreement,
    retrieval: Retrieval,
    store: Picking,
    base: Picking,
}

impl TryFrom<&[Token]> for Eval {
    type Error = Failure;

    fn try_from(rest: &[Token]) -> Result<Eval, Failure> {
        let [session, from, until] = <&[Token; 3]>::try_from(rest).map_err(|_count| Failure::Usage)?;
        Ok(Eval {
            session: SessionId::try_new(Token::only(slice::from_ref(session))?.to_owned())
                .map_err(|_bad| Failure::Usage)?,
            from: Instant::try_new(Token::only(slice::from_ref(from))?.to_owned()).map_err(|_bad| Failure::Usage)?,
            until: Instant::try_new(Token::only(slice::from_ref(until))?.to_owned()).map_err(|_bad| Failure::Usage)?,
        })
    }
}

impl Eval {
    pub fn turn_ends(&self) -> TurnEnds {
        TurnEnds::between(self.session.clone(), self.from.clone(), self.until.clone())
    }
}

impl From<Option<&Asked>> for Label {
    fn from(asked: Option<&Asked>) -> Label {
        asked.map_or(Label::Silent, |_question| Label::Asks)
    }
}

impl Called {
    pub fn new(watched: &Watched, [start, end]: [Wall; 2]) -> Called {
        watched.visited(Spanned { seconds: end.into_inner() - start.into_inner() })
    }

    pub fn reply(&self) -> Option<&Reply> {
        self.finished.as_ref().map(Finished::reply)
    }

    fn spent(&self) -> Spent {
        let tokens = self
            .finished
            .as_ref()
            .map_or(0, |finished| finished.usage().spent().iter().map(|tokens| u64::from(tokens.into_inner())).sum());
        Spent { tokens, seconds: self.seconds }
    }

    fn listed(&self) -> Vec<Uuid> {
        self.reply().map_or_default(|reply| Replied::from(reply).listed())
    }
}

impl Watching for Spanned {
    type Output = Called;

    fn finished(self, finished: &Finished) -> Called {
        Called { finished: Some(finished.clone()), seconds: self.seconds }
    }

    fn stalled(self, _stall: &Stall) -> Called {
        Called { finished: None, seconds: self.seconds }
    }
}

impl Truth {
    pub fn read(call: &Called, history: &[Said]) -> Truth {
        let known: BTreeSet<&Uuid> = history.iter().map(AsRef::<Uuid>::as_ref).collect();
        Truth { uuids: call.listed().into_iter().filter(|uuid| known.contains(uuid)).collect(), spent: call.spent() }
    }
}

impl Arm {
    pub fn picked(call: &Called, pool: &[Said]) -> Arm {
        Arm { quotes: call.reply().map_or_default(|reply| Answers::quoted(reply, pool)), spent: call.spent() }
    }

    pub fn verifying<'turn>(&'turn self, asked: &'turn Asked) -> Verifying<'turn> {
        Verifying { question: asked.as_ref(), quotes: &self.quotes }
    }

    pub fn checked(self, call: &Called) -> Checked {
        Checked { verified: call.listed().into_iter().collect(), arm: self }
    }
}

impl Checked {
    fn picking(&self, truth: &BTreeSet<Uuid>, reachable: usize) -> Picking {
        let picked: BTreeSet<&Uuid> = self.arm.quotes.iter().map(AsRef::<Uuid>::as_ref).collect();
        let found = truth.iter().filter(|uuid| picked.contains(uuid)).count();
        Picking {
            found_any: usize::from(found > 0),
            found,
            reachable,
            picks: self.arm.quotes.len(),
            verified: self
                .arm
                .quotes
                .iter()
                .filter(|quote| self.verified.contains(AsRef::<Uuid>::as_ref(*quote)))
                .count(),
            tokens: self.arm.spent.tokens,
            seconds: self.arm.spent.seconds,
        }
    }
}

impl Judged {
    const TWELVE: usize = 12;

    pub fn new(truth: Truth, candidates: &[Said], store: Checked, base: Checked) -> Judged {
        let uuids = candidates.iter().map(AsRef::<Uuid>::as_ref).cloned();
        Judged { truth, twelve: uuids.clone().take(Self::TWELVE).collect(), candidates: uuids.collect(), store, base }
    }

    fn tally(&self) -> Tally {
        let truths = &self.truth.uuids;
        let twelve = truths.intersection(&self.twelve).count();
        let all = truths.intersection(&self.candidates).count();
        Tally {
            agreement: Agreement::default(),
            retrieval: Retrieval {
                judged: 1,
                answered: usize::from(truths.is_empty().not()),
                truths: truths.len(),
                twelve_any: usize::from(twelve > 0),
                all_any: usize::from(all > 0),
                twelve_truths: twelve,
                all_truths: all,
                truth_tokens: self.truth.spent.tokens,
            },
            store: self.store.picking(truths, all),
            base: self.base.picking(truths, truths.len()),
        }
    }
}

impl Turn {
    pub fn new(end: &TurnEnd, [haiku, sonnet]: [Option<&Asked>; 2], judged: Option<Judged>) -> Turn {
        let asked = end.asked();
        Turn {
            asked: AsRef::<Uuid>::as_ref(asked).clone(),
            at: AsRef::<Instant>::as_ref(asked).clone(),
            labels: Labels { haiku: Label::from(haiku), sonnet: Label::from(sonnet) },
            judged,
        }
    }

    fn tally(&self) -> Tally {
        let judged = self.judged.as_ref().map_or_default(Judged::tally);
        Tally { agreement: Agreement::from(self.labels), ..judged }
    }
}

impl From<Labels> for Agreement {
    fn from(labels: Labels) -> Agreement {
        let haiku = usize::from(labels.haiku == Label::Asks);
        let sonnet = usize::from(labels.sonnet == Label::Asks);
        Agreement {
            both: haiku * sonnet,
            haiku: haiku * (1 - sonnet),
            sonnet: (1 - haiku) * sonnet,
            neither: (1 - haiku) * (1 - sonnet),
        }
    }
}

impl From<Vec<Turn>> for Evaluated {
    fn from(turns: Vec<Turn>) -> Evaluated {
        let tally = Tally::concat(turns.iter().map(Turn::tally));
        Evaluated { turns, tally }
    }
}

impl Semigroup for Agreement {
    fn combine(self, other: Agreement) -> Agreement {
        Agreement {
            both: self.both + other.both,
            haiku: self.haiku + other.haiku,
            sonnet: self.sonnet + other.sonnet,
            neither: self.neither + other.neither,
        }
    }
}

impl Monoid for Agreement {}

impl Semigroup for Retrieval {
    fn combine(self, other: Retrieval) -> Retrieval {
        Retrieval {
            judged: self.judged + other.judged,
            answered: self.answered + other.answered,
            truths: self.truths + other.truths,
            twelve_any: self.twelve_any + other.twelve_any,
            all_any: self.all_any + other.all_any,
            twelve_truths: self.twelve_truths + other.twelve_truths,
            all_truths: self.all_truths + other.all_truths,
            truth_tokens: self.truth_tokens + other.truth_tokens,
        }
    }
}

impl Monoid for Retrieval {}

impl Semigroup for Picking {
    fn combine(self, other: Picking) -> Picking {
        Picking {
            found_any: self.found_any + other.found_any,
            found: self.found + other.found,
            reachable: self.reachable + other.reachable,
            picks: self.picks + other.picks,
            verified: self.verified + other.verified,
            tokens: self.tokens + other.tokens,
            seconds: self.seconds + other.seconds,
        }
    }
}

impl Monoid for Picking {}

impl Semigroup for Tally {
    fn combine(self, other: Tally) -> Tally {
        Tally {
            agreement: self.agreement.combine(other.agreement),
            retrieval: self.retrieval.combine(other.retrieval),
            store: self.store.combine(other.store),
            base: self.base.combine(other.base),
        }
    }
}

impl Monoid for Tally {}

impl<'turn> Grounding<'turn> {
    pub fn new(asked: &'turn Asked, reply: Unabridged<'turn>, history: &'turn [Said]) -> Grounding<'turn> {
        Grounding { question: asked.as_ref(), reply, history }
    }
}

impl fmt::Display for Grounding<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let history: Vec<String> = self.history.iter().map(|said| Unabridged::from(said).to_string()).collect();
        write!(
            formatter,
            "An AI agent ended its turn by asking its owner this question:\n{}\n\n\
             The owner then replied:\n{}\n\n\
             Below are all of the owner's earlier messages, each headed by [uuid timestamp].\n\
             List every earlier message that had already answered the question, so that the agent could have acted without asking: the owner had already stated the decision, rule, preference or fact the question needed.\n\
             Reply with JSON only, no prose and no code fence: a list of the full uuids from the headers. Reply [] if none had.\n\n{}",
            self.question,
            self.reply,
            history.join("\n\n")
        )
    }
}

impl fmt::Display for Verifying<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let quotes: Vec<String> = self.quotes.iter().map(ToString::to_string).collect();
        write!(
            formatter,
            "An AI agent was about to ask its owner this question:\n{}\n\n\
             Each passage below, headed by its message uuid and timestamp, was offered as something the owner said earlier that answers it.\n\
             Judge each passage on its own: does it answer the question or bear directly on it?\n\
             Reply with JSON only, no prose and no code fence: a list of the uuids of the passages that do. Reply [] if none do.\n\n{}",
            self.question,
            quotes.join("\n")
        )
    }
}

impl fmt::Display for Turn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tally = self.tally();
        write!(
            formatter,
            "turn {} {} haiku={:?} sonnet={:?}",
            self.asked, self.at, self.labels.haiku, self.labels.sonnet
        )?;
        self.judged.as_ref().map_or(Ok(()), |judged| {
            let truths: Vec<String> = judged.truth.uuids.iter().map(ToString::to_string).collect();
            write!(formatter, " truth=[{}] {}", truths.join(" "), tally)
        })
    }
}

impl fmt::Display for Tally {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let retrieval = self.retrieval;
        write!(
            formatter,
            "judged={} answered={} truths={} twelve_any={} all_any={} twelve_truths={} all_truths={} truth_tokens={} store=[{}] base=[{}]",
            retrieval.judged,
            retrieval.answered,
            retrieval.truths,
            retrieval.twelve_any,
            retrieval.all_any,
            retrieval.twelve_truths,
            retrieval.all_truths,
            retrieval.truth_tokens,
            self.store,
            self.base
        )
    }
}

impl fmt::Display for Picking {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "found_any={} found={} reachable={} picks={} verified={} tokens={} seconds={}",
            self.found_any, self.found, self.reachable, self.picks, self.verified, self.tokens, self.seconds
        )
    }
}

impl fmt::Display for Evaluated {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.turns.iter().try_for_each(|turn| writeln!(formatter, "{turn}"))?;
        let agreement = self.tally.agreement;
        writeln!(
            formatter,
            "stage1 both={} haiku_only={} sonnet_only={} neither={}",
            agreement.both, agreement.haiku, agreement.sonnet, agreement.neither
        )?;
        writeln!(formatter, "total {}", self.tally)
    }
}
