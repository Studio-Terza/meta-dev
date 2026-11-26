use std::collections::BTreeSet;
use std::fmt;
use std::ops::Not;

use design_model::{Consideration, CrateName, FeatureName, Ordinal};
use store_ports::{Batch, Columns, Execute, Prepared, Query, StoreFailure, Written};
use transcript_ports::{Body, Instant, Uuid};

use super::{FeatureNamed, Featured, Features};
use crate::Quote;
use crate::ideals::Escaped;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Feature {
    name: FeatureName,
    consideration: Consideration,
    quotes: Vec<Quote>,
    crates: Vec<CrateName>,
}

#[derive(Debug, Columns)]
struct FeatureQuoteRow {
    name: FeatureName,
    consideration: Consideration,
    message: Uuid,
    at: Instant,
    excerpt: Body,
}

#[derive(Debug, Columns)]
struct MemberRow {
    feature: FeatureName,
    member: CrateName,
}

impl Feature {
    #[must_use]
    pub fn new(name: FeatureName, consideration: Consideration, quotes: Vec<Quote>, crates: Vec<CrateName>) -> Feature {
        Feature { name, consideration, quotes, crates }
    }

    #[must_use]
    pub fn crates(&self) -> &[CrateName] {
        &self.crates
    }

    #[must_use]
    pub fn absorbed(self, later: Feature) -> Feature {
        let quotes: Vec<Quote> = self.quotes.iter().chain(&later.quotes).cloned().collect();
        let kept = quotes
            .iter()
            .enumerate()
            .filter(|(index, quote)| quotes.get(..*index).unwrap_or_default().contains(quote).not())
            .map(|(_index, quote)| quote.clone())
            .collect();
        let crates: BTreeSet<CrateName> = self.crates.into_iter().chain(later.crates).collect();
        Feature { quotes: kept, crates: crates.into_iter().collect(), ..self }
    }

    fn grouped(group: &[FeatureQuoteRow], members: &[MemberRow]) -> Option<Feature> {
        group.first().map(|first| Feature {
            name: first.name.clone(),
            consideration: first.consideration,
            quotes: group.iter().map(FeatureQuoteRow::quote).collect(),
            crates: members.iter().filter(|row| row.feature == first.name).map(|row| row.member.clone()).collect(),
        })
    }

    fn inserted(&self, number: u32) -> Vec<Prepared<()>> {
        let Feature { name, consideration, quotes, crates } = self;
        let rank = Ordinal::new(number);
        let listed = store_ports::query!(
            r#"
INSERT INTO
  features (name, place, consideration)
VALUES
  (?1, ?2, ?3)
"#,
            name: FeatureName,
            rank: Ordinal,
            consideration: Consideration
        );
        let quoted = quotes.iter().zip(0_u32..).map(|(quote, counted)| {
            let place = Ordinal::new(counted);
            let message: &Uuid = quote.as_ref();
            let excerpt: &Body = quote.as_ref();
            store_ports::query!(
                r#"
INSERT INTO
  feature_quotes (feature, place, message, excerpt)
VALUES
  (?1, ?2, ?3, ?4)
"#,
                name: FeatureName,
                place: Ordinal,
                message: Uuid,
                excerpt: Body
            )
        });
        let members = crates.iter().map(|member| {
            store_ports::query!(
                r#"
INSERT
  OR IGNORE INTO feature_crates (feature, member)
VALUES
  (?1, ?2)
"#,
                name: FeatureName,
                member: CrateName
            )
        });
        [listed].into_iter().chain(quoted).chain(members).collect()
    }
}

impl AsRef<FeatureName> for Feature {
    fn as_ref(&self) -> &FeatureName {
        &self.name
    }
}

impl FeatureQuoteRow {
    fn quote(&self) -> Quote {
        Quote::new(self.message.clone(), self.at.clone(), self.excerpt.clone())
    }
}

impl Batch for Featured {
    type Failure = StoreFailure;

    fn written(self, statements: &mut impl Execute) -> Result<Written, StoreFailure> {
        let Featured(features) = self;
        let cleared = [
            store_ports::query!(
                r#"
DELETE FROM
  feature_crates
"#
            ),
            store_ports::query!(
                r#"
DELETE FROM
  feature_quotes
"#
            ),
            store_ports::query!(
                r#"
DELETE FROM
  features
"#
            ),
        ];
        let listed = features.iter().zip(0_u32..).flat_map(|(feature, number)| feature.inserted(number));
        statements.written_all(cleared.into_iter().chain(listed))
    }
}

impl Query for Features {
    type Rows = Vec<Feature>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<Feature>, StoreFailure> {
        let rows = statements.rows(store_ports::query!(
            FeatureQuoteRow,
            r#"
SELECT
  features.name,
  features.consideration,
  feature_quotes.message,
  messages.at,
  feature_quotes.excerpt
FROM
  features
  JOIN feature_quotes ON feature_quotes.feature = features.name
  JOIN messages ON messages.uuid = feature_quotes.message
ORDER BY
  features.place,
  feature_quotes.place
"#,
        ))?;
        let members = statements.rows(store_ports::query!(
            MemberRow,
            r#"
SELECT
  feature,
  member
FROM
  feature_crates
ORDER BY
  feature,
  member
"#,
        ))?;
        Ok(rows
            .chunk_by(|left, right| left.name == right.name)
            .filter_map(|group| Feature::grouped(group, &members))
            .collect())
    }
}

impl Query for FeatureNamed {
    type Rows = Option<Feature>;

    fn rows(self, statements: &mut impl Execute) -> Result<Option<Feature>, StoreFailure> {
        let FeatureNamed(name) = self;
        Ok(Features.rows(statements)?.into_iter().find(|feature| feature.name == name))
    }
}

impl fmt::Display for Feature {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let named: Vec<&str> = self.crates.iter().map(AsRef::<str>::as_ref).collect();
        let members = named.first().map_or_else(|| String::from("-"), |_first| named.join(","));
        let lines: Vec<String> = self
            .quotes
            .iter()
            .map(|quote| {
                let uuid: &Uuid = quote.as_ref();
                let excerpt: &Body = quote.as_ref();
                format!("{}\t{}\t{members}\t{uuid}\t{}", self.name, self.consideration, Escaped::from(excerpt.as_ref()))
            })
            .collect();
        write!(formatter, "{}", lines.join("\n"))
    }
}
