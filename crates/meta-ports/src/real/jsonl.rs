//! JSONL framing for the meta-dev log: exactly one record per line. Two records on a line, a
//! record split over lines and a blank line are all refused, each with its line number.
//! The side file of unverified escapes is read leniently instead: a line that cannot be read is
//! counted, never fatal, because branch builds wrote it.
//! A registered boundary (self/parse-boundaries.toml): a line is the unit that is parsed.
#![allow(clippy::disallowed_methods)]

use crate::PortError;
use meta_core::{Record, SideLine, UnverifiedEscapes};
use std::path::Path;

pub(super) fn records(path: &Path, text: &str) -> Result<Vec<Record>, PortError> {
    text.lines()
        .enumerate()
        .map(|(index, line)| {
            serde_json::from_str(line).map_err(|error| {
                let path = path.display();
                PortError::new(format!(
                    "{path} line {}: {error}. Next: meta escape <kind> <reason>, then repair {path}.",
                    index + 1
                ))
            })
        })
        .collect()
}

/// Each line of `bytes` as written, with the escape it holds when it can be read; a blank line is
/// no line at all.
pub(super) fn side_lines(bytes: &[u8]) -> Vec<SideLine> {
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| SideLine::new(line.to_vec(), serde_json::from_slice(line).ok()))
        .collect()
}

/// What the side file holds, each line that cannot be read only counted.
pub(super) fn lenient(bytes: &[u8]) -> UnverifiedEscapes {
    UnverifiedEscapes::of(side_lines(bytes).iter().map(|line| line.escape().cloned()))
}

#[cfg(test)]
mod tests {
    use super::records;
    use std::path::Path;

    const RECORD: &str = "{\"at\":1,\"event\":\"Done\",\"task\":1}";
    const NEXT: &str = "Next: meta escape <kind> <reason>, then repair log.jsonl.";

    fn refusal(text: &str) -> String {
        records(Path::new("log.jsonl"), text).unwrap_err().to_string()
    }

    #[test]
    fn a_corrupt_line_is_reported_with_its_line_number() {
        let text = format!("{RECORD}\n{{\"at\":2,\"event\":}}\n");
        assert_eq!(refusal(&text), format!("log.jsonl line 2: expected value at line 1 column 17. {NEXT}"));
        assert_eq!(records(Path::new("log.jsonl"), &format!("{RECORD}\n")).unwrap().len(), 1);
    }

    #[test]
    fn a_log_holds_exactly_one_record_per_line() {
        let two_on_one_line = format!("{RECORD}{RECORD}\n");
        let split_over_lines = "{\"at\":1,\n\"event\":\"Done\",\"task\":1}\n".to_owned();
        let blank_line = format!("{RECORD}\n\n{RECORD}\n");
        let errors = [two_on_one_line, split_over_lines, blank_line].map(|text| refusal(&text));
        assert_eq!(
            errors,
            [
                format!("log.jsonl line 1: trailing characters at line 1 column 33. {NEXT}"),
                format!("log.jsonl line 1: EOF while parsing a value at line 1 column 8. {NEXT}"),
                format!("log.jsonl line 2: EOF while parsing a value at line 1 column 0. {NEXT}"),
            ]
        );
    }
}
