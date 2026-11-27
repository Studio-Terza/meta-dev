mod boundary;

use std::collections::BTreeSet;
use std::fmt;
use std::path::PathBuf;
use std::thread::{self, ScopedJoinHandle};

use console_ports::Shown;
use design_messages::{OwnerHistory, Said, StopRow, StopTrail, Tally};
use dispatch_ports::{Clock, Dispatch};
use file_ports::{Bytes, Files};
use newtype::Wrapping;
use store_ports::Store;
use strum::VariantArray;
use transcript_ports::{Body, Uuid};

use super::Ground;
use super::crew::Crew;
use super::pairs::Pair;
use super::walk::{Tree, Walk, Walking};
use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::uttered::Uttered;
pub use boundary::{Log, Spoken};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Backtest {
    tree: PathBuf,
    session: PathBuf,
    pairs: PathBuf,
    rows: PathBuf,
    splits: Vec<Split>,
}

#[derive(strum::Display, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum Split {
    Tune,
    Test,
}

#[derive(strum::Display, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
enum Set {
    Stop,
    Pair,
}

#[derive(Debug)]
struct Case<'tree> {
    set: Set,
    split: Split,
    index: usize,
    said: Body,
    gold: BTreeSet<&'tree Uuid>,
}

#[derive(Debug)]
struct Trial<'run, 'tree> {
    case: &'run Case<'tree>,
    retrieved: BTreeSet<&'tree Uuid>,
    millis: Tally,
    tokens: Tally,
}

#[derive(Debug)]
struct Rows<'run, 'tree>(&'run [Trial<'run, 'tree>]);

#[derive(Debug)]
struct Summary<'run, 'tree> {
    set: Set,
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
    const CONCURRENT: usize = 8;
    const HEADER: &'static str = "set\tsplit\tindex\tgold\tretrieved\tfound\trecall\tmillis\ttokens\n";

    pub fn new([tree, session, pairs, rows]: [&&str; 4], splits: &[Split]) -> Backtest {
        Backtest {
            tree: PathBuf::from(tree),
            session: PathBuf::from(session),
            pairs: PathBuf::from(pairs),
            rows: PathBuf::from(rows),
            splits: splits.to_vec(),
        }
    }

    pub fn answered<F: Files, S: Store, D: Dispatch + Sync, C: Clock + Sync>(
        &self,
        ground: &Ground<'_, F, S>,
        crew: &Crew<'_, D, C>,
    ) -> Result<Uttered, Failure> {
        let files = ground.files();
        let tree = Walk::planted(files, &self.tree)?;
        let log = Log::read(files.read_text(&self.session)?.as_ref());
        let pairs = Pair::read(files.read_text(&self.pairs)?.as_ref());
        let history: Vec<Said> = ground.store().read(OwnerHistory)?;
        let brute = Brute(ground.store().read(StopTrail)?);
        let cases: Vec<Case<'_>> =
            Backtest::stops(&tree, &log).into_iter().chain(Backtest::paired(&tree, &pairs)).collect();
        let chosen: Vec<&Case<'_>> = cases.iter().filter(|case| self.splits.contains(&case.split)).collect();
        files.write(&self.rows, Bytes::from(Backtest::HEADER))?;
        let walking = Walking::new(crew);
        let batches = chosen
            .chunks(Backtest::CONCURRENT)
            .map(|batch| self.batch(files, &walking, crew, &tree, batch))
            .collect::<Result<Vec<Vec<Trial<'_, '_>>>, Failure>>()?;
        let trials: Vec<Trial<'_, '_>> = batches.into_iter().flatten().collect();
        let summaries = Set::VARIANTS
            .iter()
            .flat_map(|set| self.splits.iter().map(|split| Summary::of(*set, *split, &trials)))
            .filter(|summary| !summary.trials.is_empty())
            .collect();
        let extras = Extra::sampled(&trials, &history);
        let report = Report { trials: &trials, summaries, brute, extras };
        Ok(Uttered::new(Shown::said(report), Ok(Outcome::Backtested)))
    }

    fn batch<'run, 'tree, D: Dispatch + Sync, C: Clock + Sync>(
        &self,
        files: &impl Files,
        walking: &Walking<'_, D, C>,
        crew: &Crew<'_, D, C>,
        tree: &'tree Tree,
        batch: &[&'run Case<'tree>],
    ) -> Result<Vec<Trial<'run, 'tree>>, Failure> {
        let trials = thread::scope(|scope| {
            Trial::joined(batch.iter().map(|case| scope.spawn(|| Trial::run(walking, crew, tree, case))).collect())
        })?;
        files.append(&self.rows, &Bytes::from(&Rows(&trials)))?;
        Ok(trials)
    }

    fn stops<'tree>(tree: &'tree Tree, log: &Log) -> Vec<Case<'tree>> {
        let known = tree.messages();
        let stops = log.stops();
        stops
            .iter()
            .enumerate()
            .map(|(index, stop)| Case {
                set: Set::Stop,
                split: Split::at(index, stops.len()),
                index,
                said: stop.said().clone(),
                gold: known.uuids().iter().filter_map(|uuid| stop.quoted(uuid)).collect(),
            })
            .collect()
    }

    fn paired<'tree>(tree: &'tree Tree, pairs: &[Pair]) -> Vec<Case<'tree>> {
        let known = tree.messages();
        let kept: Vec<(&Pair, &'tree Uuid)> =
            pairs.iter().filter_map(|pair| known.uuids().get(pair.answer()).map(|uuid| (pair, *uuid))).collect();
        kept.iter()
            .enumerate()
            .map(|(index, (pair, uuid))| Case {
                set: Set::Pair,
                split: Split::at(index, kept.len()),
                index,
                said: pair.question().clone(),
                gold: BTreeSet::from([*uuid]),
            })
            .collect()
    }
}

impl Split {
    pub fn named(word: &str) -> Result<Split, Failure> {
        Split::VARIANTS.iter().find(|split| split.to_string() == word).copied().ok_or(Failure::Usage)
    }

    fn at(index: usize, count: usize) -> Split {
        if index < count.saturating_mul(Backtest::TUNED) / Backtest::TENTHS { Split::Tune } else { Split::Test }
    }
}

impl<'run, 'tree> Trial<'run, 'tree> {
    fn run<D: Dispatch, C: Clock>(
        walking: &Walking<'_, D, C>,
        crew: &Crew<'_, D, C>,
        tree: &'tree Tree,
        case: &'run Case<'tree>,
    ) -> Result<Trial<'run, 'tree>, Failure> {
        let started = crew.moment();
        let walked = walking.walked(tree, &case.said)?;
        let millis = Tally::new(crew.moment().since(started).max(0));
        Ok(Trial { case, retrieved: walked.retrieved().uuids().clone(), millis, tokens: walked.tokens() })
    }

    fn joined(
        running: Vec<ScopedJoinHandle<'_, Result<Trial<'run, 'tree>, Failure>>>,
    ) -> Result<Vec<Trial<'run, 'tree>>, Failure> {
        running.into_iter().map(|handle| handle.join().map_err(|_panicked| Failure::Panicked)?).collect()
    }

    fn found(&self) -> usize {
        self.case.gold.intersection(&self.retrieved).count()
    }

    fn recall(&self) -> Permille {
        Permille::of(self.found(), self.case.gold.len())
    }

    fn extras(&self) -> impl Iterator<Item = &&'tree Uuid> {
        self.retrieved.difference(&self.case.gold)
    }
}

impl<'run, 'tree> Summary<'run, 'tree> {
    const PERCENTILE: usize = 90;
    const WHOLE: usize = 100;

    fn of(set: Set, split: Split, trials: &'run [Trial<'run, 'tree>]) -> Summary<'run, 'tree> {
        let kept = trials.iter().filter(|trial| trial.case.set == set && trial.case.split == split).collect();
        Summary { set, split, trials: kept }
    }

    fn median<T: Ord + Copy + Default>(values: Vec<T>) -> T {
        Summary::ranked(values, 1, 2)
    }

    fn ranked<T: Ord + Copy + Default>(mut values: Vec<T>, part: usize, whole: usize) -> T {
        values.sort_unstable();
        let at = values.len().saturating_mul(part).checked_div(whole).unwrap_or_default();
        values.get(at).copied().unwrap_or_default()
    }
}

impl<'run> Extra<'run> {
    fn sampled<'tree: 'run>(trials: &'run [Trial<'run, 'tree>], history: &'run [Said]) -> Vec<Extra<'run>> {
        let stops: Vec<&Trial<'_, '_>> = trials.iter().filter(|trial| trial.case.set == Set::Stop).collect();
        let tested = stops.iter().any(|trial| trial.case.split == Split::Test);
        let sampled: Vec<&Trial<'_, '_>> = stops
            .into_iter()
            .filter(|trial| tested.then_some(Split::Test).is_none_or(|split| trial.case.split == split))
            .filter(|trial| trial.extras().next().is_some())
            .collect();
        let step = sampled.len().div_ceil(Backtest::EXTRAS).max(1);
        sampled
            .iter()
            .step_by(step)
            .filter_map(|trial| trial.extras().next().map(|uuid| (trial, *uuid)))
            .map(|(trial, uuid)| Extra {
                split: trial.case.split,
                index: trial.case.index,
                uuid,
                stop: &trial.case.said,
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
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.case.set,
            self.case.split,
            self.case.index,
            self.case.gold.len(),
            self.retrieved.len(),
            self.found(),
            self.recall(),
            self.millis,
            self.tokens
        )
    }
}

impl fmt::Display for Rows<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|trial| writeln!(formatter, "{trial}"))
    }
}

impl fmt::Display for Summary<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let recalls: Vec<Permille> = self.trials.iter().map(|trial| trial.recall()).collect();
        let full = recalls.iter().filter(|recall| recall.0 == Permille::WHOLE).count();
        let retrieved = Summary::median(self.trials.iter().map(|trial| trial.retrieved.len()).collect());
        let millis: Vec<i64> = self.trials.iter().map(|trial| trial.millis.into_inner()).collect();
        let tokens: Tally = self.trials.iter().map(|trial| trial.tokens).sum();
        write!(
            formatter,
            "summary\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.set,
            self.split,
            self.trials.len(),
            Permille::mean(&recalls),
            Permille::of(full, self.trials.len()),
            retrieved,
            Summary::median(millis.clone()),
            Summary::ranked(millis, Summary::PERCENTILE, Summary::WHOLE),
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
        let millis: Vec<i64> = spent.iter().map(|[millis, _tokens]| *millis).collect();
        let tokens = Summary::median(spent.iter().map(|[_millis, tokens]| *tokens).collect());
        write!(
            formatter,
            "brute\t{}\t{}\t{}\t{tokens}",
            spent.len(),
            Summary::median(millis.clone()),
            Summary::ranked(millis, Summary::PERCENTILE, Summary::WHOLE)
        )
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
        write!(formatter, "{}{}", Backtest::HEADER, Rows(self.trials))?;
        writeln!(formatter, "summary\tset\tsplit\tcases\trecall\tfull\tretrieved\tmillis\tp90\ttokens")?;
        self.summaries.iter().try_for_each(|summary| writeln!(formatter, "{summary}"))?;
        writeln!(formatter, "brute\tstops\tmillis\tp90\ttokens")?;
        writeln!(formatter, "{}", self.brute)?;
        writeln!(formatter, "extra\tsplit\tindex\tuuid\tstop\tmessage")?;
        self.extras.iter().try_for_each(|extra| writeln!(formatter, "{extra}"))
    }
}
