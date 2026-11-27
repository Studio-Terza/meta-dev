mod boundary;

use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;

use console_ports::Shown;
use design_messages::{OwnerHistory, Said, StopRow, StopTrail, Tally};
use dispatch_ports::{Clock, Dispatch};
use file_ports::Files;
use newtype::Wrapping;
use store_ports::Store;
use strum::VariantArray;
use transcript_ports::{Body, Uuid};

use super::Ground;
use super::crew::Crew;
use super::walk::{Tree, Walk, Walking};
use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
use boundary::Logged;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Backtest {
    tree: PathBuf,
    session: PathBuf,
    splits: Vec<Split>,
}

#[derive(strum::Display, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum Split {
    Tune,
    Test,
}

#[derive(Debug)]
struct Stop<'tree> {
    split: Split,
    index: usize,
    said: Body,
    gold: BTreeSet<&'tree Uuid>,
}

#[derive(Debug)]
struct Trial<'run, 'tree> {
    stop: &'run Stop<'tree>,
    retrieved: BTreeSet<&'tree Uuid>,
    millis: Tally,
    tokens: Tally,
}

#[derive(Debug)]
struct Summary<'run, 'tree> {
    split: Split,
    trials: Vec<&'run Trial<'run, 'tree>>,
}

#[derive(Debug)]
struct Brute(Vec<StopRow>);

#[derive(Debug)]
struct Extra<'run> {
    split: Split,
    index: usize,
    uuid: &'run Uuid,
    stop: &'run Body,
    message: Option<&'run Body>,
}

#[derive(Debug)]
struct Report<'run, 'tree> {
    trials: &'run [Trial<'run, 'tree>],
    summaries: Vec<Summary<'run, 'tree>>,
    brute: Brute,
    extras: Vec<Extra<'run>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Permille(usize);

impl Backtest {
    const TUNED: usize = 7;
    const TENTHS: usize = 10;
    const EXTRAS: usize = 10;

    pub fn new([tree, session]: [&&str; 2], splits: &[Split]) -> Backtest {
        Backtest { tree: PathBuf::from(tree), session: PathBuf::from(session), splits: splits.to_vec() }
    }

    pub fn answered<F: Files, S: Store, D: Dispatch, C: Clock>(
        &self,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
    ) -> Result<Uttered, Failure> {
        let tree = Walk::planted(ground.files(), &self.tree)?;
        let logged = Logged::read(ground.files().read_text(&self.session)?.as_ref());
        let history: Vec<Said> = ground.store().read(OwnerHistory)?;
        let brute = Brute(ground.store().read(StopTrail)?);
        let stops = Backtest::stops(&tree, &logged);
        let walking = Walking::new(crew);
        let trials = stops
            .iter()
            .filter(|stop| self.splits.contains(&stop.split))
            .map(|stop| Trial::run(&walking, crew, &tree, stop))
            .collect::<Result<Vec<Trial<'_, '_>>, Failure>>()?;
        let summaries = self.splits.iter().map(|split| Summary::of(*split, &trials)).collect();
        let extras = Extra::sampled(&trials, &history);
        let report = Report { trials: &trials, summaries, brute, extras };
        Ok(Uttered::new(Shown::said(report), Ok(Outcome::Backtested)))
    }

    fn stops<'tree>(tree: &'tree Tree, logged: &[Logged]) -> Vec<Stop<'tree>> {
        let known = tree.messages();
        let tuned = logged.len().saturating_mul(Backtest::TUNED) / Backtest::TENTHS;
        logged
            .iter()
            .enumerate()
            .map(|(index, stop)| Stop {
                split: if index < tuned { Split::Tune } else { Split::Test },
                index,
                said: stop.said().clone(),
                gold: known.uuids().iter().filter_map(|uuid| stop.quoted(uuid)).collect(),
            })
            .collect()
    }
}

impl Split {
    pub fn named(word: &str) -> Result<Split, Failure> {
        Split::VARIANTS.iter().find(|split| split.to_string() == word).copied().ok_or(Failure::Usage)
    }
}

impl<'run, 'tree> Trial<'run, 'tree> {
    fn run<D: Dispatch, C: Clock>(
        walking: &Walking<'_, D, C>,
        crew: &Crew<'_, D, C>,
        tree: &'tree Tree,
        stop: &'run Stop<'tree>,
    ) -> Result<Trial<'run, 'tree>, Failure> {
        let started = crew.moment();
        let walked = walking.walked(tree, &stop.said)?;
        let millis = Tally::new(crew.moment().since(started).max(0));
        Ok(Trial { stop, retrieved: walked.retrieved().uuids().clone(), millis, tokens: walked.tokens() })
    }

    fn found(&self) -> usize {
        self.stop.gold.intersection(&self.retrieved).count()
    }

    fn recall(&self) -> Permille {
        Permille::of(self.found(), self.stop.gold.len())
    }

    fn extras(&self) -> impl Iterator<Item = &&'tree Uuid> {
        self.retrieved.difference(&self.stop.gold)
    }
}

impl<'run, 'tree> Summary<'run, 'tree> {
    fn of(split: Split, trials: &'run [Trial<'run, 'tree>]) -> Summary<'run, 'tree> {
        Summary { split, trials: trials.iter().filter(|trial| trial.stop.split == split).collect() }
    }

    fn median<T: Ord + Copy + Default>(mut values: Vec<T>) -> T {
        values.sort_unstable();
        values.get(values.len() / 2).copied().unwrap_or_default()
    }
}

impl<'run> Extra<'run> {
    fn sampled<'tree: 'run>(trials: &'run [Trial<'run, 'tree>], history: &'run [Said]) -> Vec<Extra<'run>> {
        let tested = trials.iter().any(|trial| trial.stop.split == Split::Test);
        let sampled: Vec<&Trial<'_, '_>> = trials
            .iter()
            .filter(|trial| tested.then_some(Split::Test).is_none_or(|split| trial.stop.split == split))
            .filter(|trial| trial.extras().next().is_some())
            .collect();
        let step = sampled.len().div_ceil(Backtest::EXTRAS).max(1);
        sampled
            .iter()
            .step_by(step)
            .filter_map(|trial| trial.extras().next().map(|uuid| (trial, *uuid)))
            .map(|(trial, uuid)| Extra {
                split: trial.stop.split,
                index: trial.stop.index,
                uuid,
                stop: &trial.stop.said,
                message: history.iter().find(|said| AsRef::<Uuid>::as_ref(*said) == uuid).map(AsRef::as_ref),
            })
            .take(Backtest::EXTRAS)
            .collect()
    }
}

impl Permille {
    const WHOLE: usize = 1000;

    fn of(part: usize, whole: usize) -> Permille {
        Permille(part.saturating_mul(Permille::WHOLE).checked_div(whole).unwrap_or(Permille::WHOLE))
    }

    fn mean(values: &[Permille]) -> Permille {
        Permille(values.iter().map(|value| value.0).sum::<usize>().checked_div(values.len()).unwrap_or_default())
    }
}

impl fmt::Display for Permille {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{:03}", self.0 / Permille::WHOLE, self.0 % Permille::WHOLE)
    }
}

impl fmt::Display for Trial<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "stop\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.stop.split,
            self.stop.index,
            self.stop.gold.len(),
            self.retrieved.len(),
            self.found(),
            self.recall(),
            self.millis,
            self.tokens
        )
    }
}

impl fmt::Display for Summary<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let recalls: Vec<Permille> = self.trials.iter().map(|trial| trial.recall()).collect();
        let full = recalls.iter().filter(|recall| recall.0 == Permille::WHOLE).count();
        let retrieved = Summary::median(self.trials.iter().map(|trial| trial.retrieved.len()).collect());
        let millis = Summary::median(self.trials.iter().map(|trial| trial.millis.into_inner()).collect());
        let tokens: Tally = self.trials.iter().map(|trial| trial.tokens).sum();
        write!(
            formatter,
            "summary\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.split,
            self.trials.len(),
            Permille::mean(&recalls),
            Permille::of(full, self.trials.len()),
            retrieved,
            millis,
            tokens
        )
    }
}

impl fmt::Display for Brute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spent: Vec<[i64; 2]> = self
            .0
            .iter()
            .map(|row| row.spent().map(Wrapping::into_inner))
            .filter(|[_millis, tokens]| *tokens > 0)
            .collect();
        let millis = Summary::median(spent.iter().map(|[millis, _tokens]| *millis).collect());
        let tokens = Summary::median(spent.iter().map(|[_millis, tokens]| *tokens).collect());
        write!(formatter, "brute\t{}\t{millis}\t{tokens}", spent.len())
    }
}

impl fmt::Display for Extra<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = self.message.map_or_default(ToString::to_string);
        write!(
            formatter,
            "extra\t{}\t{}\t{}\t{}\t{}",
            self.split,
            self.index,
            self.uuid,
            serde_json::Value::from(self.stop.to_string()),
            serde_json::Value::from(message)
        )
    }
}

impl fmt::Display for Report<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "stop\tsplit\tindex\tgold\tretrieved\tfound\trecall\tmillis\ttokens")?;
        self.trials.iter().try_for_each(|trial| writeln!(formatter, "{trial}"))?;
        writeln!(formatter, "summary\tsplit\tstops\trecall\tfull\tretrieved\tmillis\ttokens")?;
        self.summaries.iter().try_for_each(|summary| writeln!(formatter, "{summary}"))?;
        writeln!(formatter, "brute\tstops\tmillis\ttokens")?;
        writeln!(formatter, "{}", self.brute)?;
        writeln!(formatter, "extra\tsplit\tindex\tuuid\tstop\tmessage")?;
        self.extras.iter().try_for_each(|extra| writeln!(formatter, "{extra}"))
    }
}
