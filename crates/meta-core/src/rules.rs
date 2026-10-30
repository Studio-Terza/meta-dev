use crate::spec::{SelfFile, Unreadable};
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    rule: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rule {
    id: String,
    text: String,
    source: String,
}

impl Rules {
    pub fn parse(text: &str) -> Result<Self, Unreadable> {
        SelfFile::Rules.parse(text)
    }
}

impl fmt::Display for Rules {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.rule
            .iter()
            .try_for_each(|rule| writeln!(formatter, "  - {}: {}\n    source: {}", rule.id, rule.text, rule.source))
    }
}

#[cfg(test)]
mod tests {
    use super::Rules;

    #[test]
    fn the_repo_rules_parse_and_include_red_first_and_the_review_rule() {
        let rules = Rules::parse(include_str!("../../../self/rules.toml")).unwrap();
        let ids: Vec<&str> = rules.rule.iter().map(|rule| rule.id.as_str()).collect();
        assert!(ids.contains(&"red-first") && ids.contains(&"review-gates"), "{ids:?}");
    }

    #[test]
    fn the_red_first_rule_allows_one_bounded_ci_wait_for_the_red_commit() {
        let rules = Rules::parse(include_str!("../../../self/rules.toml")).unwrap();
        let red_first = rules.rule.iter().find(|rule| rule.id == "red-first").unwrap();
        let wait = "push the red commit first; waiting for its CI run is the one allowed CI wait \
                    (poll every 60 s, at most 20 min); then push the fix.";
        assert!(red_first.text.contains(wait), "{}", red_first.text);
    }
}
