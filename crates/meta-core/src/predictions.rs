use crate::spec::{SelfFile, Unreadable};
use serde::Deserialize;
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
    id: String,
    hypothesis: String,
    measure: String,
    expected: String,
    falsified_by: String,
    /// The falsifier as numbers: it holds when any of these measures is above its bound.
    falsifier: BTreeMap<Measure, Bound>,
}

/// A measure meta computes from the log and GitHub.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Measure {
    /// Merges outside meta, less the escapes of a bypass kind that record them.
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

    /// One line per prediction: observed values, the registered expectation, and FALSIFIED when the falsifier holds.
    pub(crate) fn lines(&self, observe: impl Fn(Measure) -> Observation) -> Vec<String> {
        self.prediction
            .iter()
            .map(|prediction| {
                let observed: Vec<(Measure, Bound, Observation)> =
                    prediction.falsifier.iter().map(|(measure, bound)| (*measure, *bound, observe(*measure))).collect();
                let values: Vec<String> = observed
                    .iter()
                    .map(|(measure, _, observation)| match observation {
                        Ok(value) => format!("{measure} {value}"),
                        Err(why) => format!("{measure} unavailable ({why})"),
                    })
                    .collect();
                let holds = observed.iter().any(|(_, bound, observation)| {
                    observation.as_ref().is_ok_and(|value| value.doubled > bound.above * 2)
                });
                let undecided = observed.iter().any(|(_, _, observation)| observation.is_err());
                let judgement = match (holds, undecided) {
                    (true, _) => "FALSIFIED",
                    (false, true) => "UNDECIDED",
                    (false, false) => "holds",
                };
                format!(
                    "{} {}: {judgement}; observed {}; expected {}; falsified by {}; measure: {}",
                    prediction.id,
                    prediction.hypothesis,
                    values.join(", "),
                    prediction.expected,
                    prediction.falsified_by,
                    prediction.measure
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{Predictions, Quantity};

    #[test]
    fn the_repo_predictions_parse_and_hold_p1_p2_and_p6() {
        let predictions = Predictions::parse(include_str!("../../../self/predictions.toml")).unwrap();
        let ids: Vec<&str> = predictions.prediction.iter().map(|prediction| prediction.id.as_str()).collect();
        assert_eq!(ids, ["P1", "P2", "P6"]);
        assert!(predictions.prediction.iter().all(|prediction| !prediction.falsifier.is_empty()));
    }

    #[test]
    fn a_median_takes_the_middle_value_or_halfway_between_the_two_middles() {
        let medians = [vec![], vec![4], vec![3, 1, 2], vec![1, 2, 3, 4], vec![2, 2]]
            .map(|values| Quantity::median(values).map(|median| median.to_string()));
        assert_eq!(medians, [None, Some("4".into()), Some("2".into()), Some("2.5".into()), Some("2".into())]);
    }
}
