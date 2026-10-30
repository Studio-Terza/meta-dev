//! JSONL framing for the meta-dev log: exactly one record per line. Two records on a line, a
//! record split over lines and a blank line are all refused, each with its line number.
#![allow(clippy::disallowed_methods)]

use crate::PortError;
use meta_core::Record;
use std::path::Path;

pub(super) fn records(path: &Path, text: &str) -> Result<Vec<Record>, PortError> {
    text.lines()
        .enumerate()
        .map(|(index, line)| {
            serde_json::from_str(line).map_err(|error| {
                let path = path.display();
                PortError::new(format!(
                    "{path} line {}: {error}. Next: meta escape <reason>, then repair {path}.",
                    index + 1
                ))
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::records;
    use std::path::Path;

    const RECORD: &str = "{\"at\":1,\"event\":\"Done\",\"task\":1}";
    const NEXT: &str = "Next: meta escape <reason>, then repair log.jsonl.";

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
