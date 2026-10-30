//! A registry `source` is `owner YYYY-MM-DD: <words>` or `failure: <words>`; this is the one
//! place that text is taken apart.
#![allow(clippy::disallowed_methods)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(try_from = "String")]
pub struct Source;

impl TryFrom<String> for Source {
    type Error = String;

    fn try_from(text: String) -> Result<Self, String> {
        let shape = "source must be `owner YYYY-MM-DD: <words>` or `failure: <words>`";
        let Some((head, words)) = text.split_once(": ") else {
            return Err(format!("{text:?}: {shape}"));
        };
        let owner_date = head.strip_prefix("owner ").is_some_and(date);
        match (owner_date || head == "failure") && !words.trim().is_empty() {
            true => Ok(Self),
            false => Err(format!("{text:?}: {shape}")),
        }
    }
}

fn date(text: &str) -> bool {
    let parts: Vec<&str> = text.split('-').collect();
    let numbers = parts.iter().all(|part| part.chars().all(|digit| digit.is_ascii_digit()));
    let lengths: Vec<usize> = parts.iter().map(|part| part.len()).collect();
    let in_range = |part: Option<&&str>, last: u8| {
        part.and_then(|part| part.parse::<u8>().ok()).is_some_and(|number| (1..=last).contains(&number))
    };
    numbers && lengths == [4, 2, 2] && in_range(parts.get(1), 12) && in_range(parts.get(2), 31)
}

#[cfg(test)]
mod tests {
    use super::Source;

    #[test]
    fn only_owner_dated_or_failure_sources_parse() {
        let parsed = [
            "owner 2026-10-30: text is parsed once",
            "failure: an agent leaked an email",
            "owner 2026-10: no day",
            "owner 2026-13-01: no such month",
            "owner 2026-10-30: ",
            "owner 26-10-30: short year",
            "failure:",
            "someone 2026-10-30: words",
            "",
        ]
        .map(|text| Source::try_from(text.to_owned()).is_ok());
        assert_eq!(parsed, [true, true, false, false, false, false, false, false, false]);
    }
}
