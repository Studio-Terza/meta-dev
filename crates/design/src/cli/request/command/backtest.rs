mod boundary;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::iter;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
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
    split: Split,
    set: Set,
    every: usize,
    offset: usize,
}

#[derive(strum::Display, VariantArray, Clone, Copy, Debug, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
enum Split {
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
    summaries: [Summary<'run, 'tree>; 1],
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

    pub fn spoken([tree, session, pairs, rows, split, set, every, offset]: [&&str; 8]) -> Result<Backtest, Failure> {
        Ok(Backtest {
            tree: PathBuf::from(tree),
            session: PathBuf::from(session),
            pairs: PathBuf::from(pairs),
            rows: PathBuf::from(rows),
            split: Split::named(split)?,
            set: Set::named(set)?,
            every: every.parse::<usize>().map_err(|_unparsed| Failure::Usage)?,
            offset: offset.parse::<usize>().map_err(|_unparsed| Failure::Usage)?,
        })
    }

    pub fn answered<F: Files + Sync, S: Store, D: Dispatch + Sync, C: Clock + Sync>(
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
        let chosen: Vec<&Case<'_>> = cases
            .iter()
            .filter(|case| case.set == self.set && case.split == self.split)
            .filter(|case| case.index.checked_rem(self.every).is_none_or(|left| left == self.offset))
            .collect();
        files.write(&self.rows, Bytes::from(Backtest::HEADER))?;
        let walking = Walking::new(crew);
        let next = AtomicUsize::new(0);
        let trials = thread::scope(|scope| {
            Trial::joined(
                iter::repeat_with(|| scope.spawn(|| self.worked(files, &walking, crew, &tree, &chosen, &next)))
                    .take(Backtest::CONCURRENT)
                    .collect(),
            )
        })?;
        let summaries = [Summary { set: self.set, split: self.split, trials: trials.iter().collect() }];
        let extras = Extra::sampled(&trials, &history);
        let report = Report { trials: &trials, summaries, brute, extras };
        Ok(Uttered::new(Shown::said(report), Ok(Outcome::Backtested)))
    }

    fn worked<'run, 'tree, D: Dispatch + Sync, C: Clock + Sync>(
        &self,
        files: &impl Files,
        walking: &Walking<'_, D, C>,
        crew: &Crew<'_, D, C>,
        tree: &'tree Tree,
        chosen: &[&'run Case<'tree>],
        next: &AtomicUsize,
    ) -> Result<Vec<Trial<'run, 'tree>>, Failure> {
        iter::from_fn(|| chosen.get(next.fetch_add(1, Ordering::Relaxed)))
            .map(|case| {
                let trial = Trial::run(walking, crew, tree, case)?;
                files.append(&self.rows, &Bytes::from(&format_args!("{trial}\n")))?;
                Ok(trial)
            })
            .collect()
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

impl Set {
    fn named(word: &str) -> Result<Set, Failure> {
        Set::VARIANTS.iter().find(|set| set.to_string() == word).copied().ok_or(Failure::Usage)
    }
}

impl Split {
    fn named(word: &str) -> Result<Split, Failure> {
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
        running: Vec<ScopedJoinHandle<'_, Result<Vec<Trial<'run, 'tree>>, Failure>>>,
    ) -> Result<Vec<Trial<'run, 'tree>>, Failure> {
        let worked = running
            .into_iter()
            .map(|handle| handle.join().map_err(|_panicked| Failure::Panicked)?)
            .collect::<Result<Vec<Vec<Trial<'run, 'tree>>>, Failure>>()?;
        let ordered: BTreeMap<usize, Trial<'run, 'tree>> =
            worked.into_iter().flatten().map(|trial| (trial.case.index, trial)).collect();
        Ok(ordered.into_values().collect())
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

impl Summary<'_, '_> {
    const PERCENTILE: usize = 90;
    const WHOLE: usize = 100;

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
