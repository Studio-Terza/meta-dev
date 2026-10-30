use crate::ids::{Observed, PredictionId};
use crate::spec::{SelfFile, Unreadable};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// The measurable predictions in `self/predictions.toml`, as registered in `docs/prereg.md`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Predictions {
    prediction: Vec<Prediction>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Prediction {
    id: PredictionId,
    hypothesis: String,
    measure: String,
    expected: String,
    falsified_by: String,
    /// Words of `docs/prereg.md` and what meta reads them as, such as reason: escape kind.
    #[serde(default)]
    terms: BTreeMap<String, String>,
    /// The falsifier as numbers: it holds when any of these measures is above its bound. Without
    /// one, the prediction is undecided.
    falsifier: Option<Falsifier>,
}

/// One or more measures, each with the bound it must stay at or under.
#[derive(Debug, Deserialize)]
#[serde(try_from = "BTreeMap<Measure, Bound>")]
struct Falsifier(BTreeMap<Measure, Bound>);

impl TryFrom<BTreeMap<Measure, Bound>> for Falsifier {
    type Error = &'static str;

    fn try_from(bounds: BTreeMap<Measure, Bound>) -> Result<Self, Self::Error> {
        match bounds.is_empty() {
            true => Err("a falsifier names at least one measure"),
            false => Ok(Self(bounds)),
        }
    }
}

impl Falsifier {
    /// FALSIFIED when any measure is above its bound, else UNDECIDED when any is unavailable, else holds.
    /// Also the readings of the measures above their bounds.
    fn judge(&self, observe: impl Fn(Measure) -> Observation) -> (&'static str, String, Vec<(Measure, Observed)>) {
        let observed: Vec<(Measure, Bound, Observation)> =
            self.0.iter().map(|(measure, bound)| (*measure, *bound, observe(*measure))).collect();
        let values: Vec<String> = observed
            .iter()
            .map(|(measure, _, observation)| match observation {
                Ok(value) => format!("{measure} {value}"),
                Err(why) => format!("{measure} unavailable ({why})"),
            })
            .collect();
        let above: Vec<(Measure, Observed)> = observed
            .iter()
            .filter(|(_, bound, observation)| observation.as_ref().is_ok_and(|value| value.doubled > bound.above * 2))
            .filter_map(|(measure, _, observation)| {
                let value = observation.as_ref().ok()?;
                Observed::try_from(format!("{measure} {value}")).ok().map(|reading| (*measure, reading))
            })
            .collect();
        let holds = observed
            .iter()
            .any(|(_, bound, observation)| observation.as_ref().is_ok_and(|value| value.doubled > bound.above * 2));
        let undecided = observed.iter().any(|(_, _, observation)| observation.is_err());
        let judgement = match (holds, undecided) {
            (true, _) => "FALSIFIED",
            (false, true) => "UNDECIDED",
            (false, false) => "holds",
        };
        (judgement, format!("observed {}", values.join(", ")), above)
    }

    #[cfg(test)]
    fn bounds(&self) -> Vec<(Measure, u64)> {
        self.0.iter().map(|(measure, bound)| (*measure, bound.above)).collect()
    }
}

/// A measure meta computes from the log and GitHub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Measure {
    /// PRs merged outside meta that no escape of a kind naming merged PRs names.
    UnrecordedBypasses,
    OpenEscapes,
    /// The most repeats of one escape kind: its escapes after the first.
    RepeatsOfOneKind,
    MedianCiRuns,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
struct Bound {
    above: u64,
}

/// A measured value, kept doubled so that a median of two middles stays exact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Quantity {
    doubled: u64,
}

impl Quantity {
    pub fn count(count: usize) -> Self {
        Self { doubled: count as u64 * 2 }
    }

    /// The median, or nothing for no values.
    pub fn median(mut values: Vec<u64>) -> Option<Self> {
        values.sort_unstable();
        let middle = values.len() / 2;
        match values.len() % 2 {
            _ if values.is_empty() => None,
            1 => values.get(middle).map(|value| Self { doubled: value * 2 }),
            _ => values.get(middle - 1).zip(values.get(middle)).map(|(low, high)| Self { doubled: low + high }),
        }
    }
}

impl fmt::Display for Quantity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.doubled % 2 {
            0 => write!(formatter, "{}", self.doubled / 2),
            _ => write!(formatter, "{}.5", self.doubled / 2),
        }
    }
}

impl fmt::Display for Measure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnrecordedBypasses => "unrecorded bypasses",
            Self::OpenEscapes => "open escapes",
            Self::RepeatsOfOneKind => "most repeats of one escape kind",
            Self::MedianCiRuns => "median CI runs per merged PR",
        })
    }
}

/// What a measure came to: a value, or why it cannot be decided now.
pub type Observation = Result<Quantity, String>;

impl Predictions {
    pub fn parse(text: &str) -> Result<Self, Unreadable> {
        SelfFile::Predictions.parse(text)
    }

    /// Each prediction read: its line (observed values, the registered expectation, FALSIFIED when the
    /// falsifier holds) and the measures above their bounds.
    pub(crate) fn read(&self, observe: impl Fn(Measure) -> Observation) -> Vec<Reading> {
        self.prediction
            .iter()
            .map(|prediction| {
                let (judgement, observed, falsifying) = match &prediction.falsifier {
                    Some(falsifier) => falsifier.judge(&observe),
                    None => ("UNDECIDED", "no falsifier registered".to_owned(), Vec::new()),
                };
                let terms: String =
                    prediction.terms.iter().map(|(word, reading)| format!("; read {word} as {reading}")).collect();
                let line = format!(
                    "{} {}: {judgement}; {observed}; expected {}; falsified by {}; measure: {}{terms}",
                    prediction.id,
                    prediction.hypothesis,
                    prediction.expected,
                    prediction.falsified_by,
                    prediction.measure
                );
                Reading { id: prediction.id.clone(), line, falsifying }
            })
            .collect()
    }
}

/// One prediction's line in `meta status`, and the measures that break their bounds now.
#[derive(Debug)]
pub(crate) struct Reading {
    pub(crate) id: PredictionId,
    pub(crate) line: String,
    pub(crate) falsifying: Vec<(Measure, Observed)>,
}

#[cfg(test)]
mod tests {
    use super::{Measure, Predictions, Quantity};

    const REPO: &str = include_str!("../../../self/predictions.toml");
    const PREREG: &str = include_str!("../../../docs/prereg.md");

    #[test]
    fn the_repo_predictions_parse_and_hold_p1_p2_and_p6() {
        let predictions = Predictions::parse(REPO).unwrap();
        let ids: Vec<String> = predictions.prediction.iter().map(|prediction| prediction.id.to_string()).collect();
        assert_eq!(ids, ["P1", "P2", "P6"]);
    }

    #[test]
    fn the_repo_prediction_bounds_are_the_numbers_registered_in_prereg() {
        let predictions = Predictions::parse(REPO).unwrap();
        let bounds: Vec<(String, Vec<(Measure, u64)>)> = predictions
            .prediction
            .iter()
            .map(|prediction| {
                let falsifier = prediction.falsifier.as_ref().map(|falsifier| falsifier.bounds()).unwrap_or_default();
                (prediction.id.to_string(), falsifier)
            })
            .collect();
        let registered = [
            ("P1", vec![(Measure::UnrecordedBypasses, 0)]),
            ("P2", vec![(Measure::OpenEscapes, 5), (Measure::RepeatsOfOneKind, 1)]),
            ("P6", vec![(Measure::MedianCiRuns, 3)]),
        ];
        let registered: Vec<(String, Vec<(Measure, u64)>)> =
            registered.into_iter().map(|(id, bounds)| (id.to_owned(), bounds)).collect();
        assert_eq!(bounds, registered);
    }

    #[test]
    fn the_repo_predictions_quote_their_prereg_row_verbatim() {
        let predictions = Predictions::parse(REPO).unwrap();
        predictions.prediction.iter().for_each(|prediction| {
            let row = format!(
                "| {} | {} | {} | {} | {} |",
                prediction.id, prediction.hypothesis, prediction.measure, prediction.expected, prediction.falsified_by
            );
            assert!(PREREG.contains(&row), "{row}");
        });
    }

    #[test]
    fn an_empty_falsifier_does_not_parse() {
        let empty = "[[prediction]]\nid = \"P9\"\nhypothesis = \"h\"\nmeasure = \"m\"\nexpected = \"e\"\n\
                     falsified_by = \"f\"\nfalsifier = {}\n";
        let error = Predictions::parse(empty).unwrap_err().to_string();
        assert!(error.contains("a falsifier names at least one measure"), "{error}");
    }

    #[test]
    fn a_median_takes_the_middle_value_or_halfway_between_the_two_middles() {
        let medians = [vec![], vec![4], vec![3, 1, 2], vec![1, 2, 3, 4], vec![2, 2]]
            .map(|values| Quantity::median(values).map(|median| median.to_string()));
        assert_eq!(medians, [None, Some("4".into()), Some("2".into()), Some("2.5".into()), Some("2".into())]);
    }
}
