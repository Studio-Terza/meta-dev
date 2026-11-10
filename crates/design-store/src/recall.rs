mod boundary;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use design_model::{NodeId, Stage, Title};
use grammar_search::{Term, Terms};
use newtype::Wrapping;
use port_core::Op;
use store_ports::{Query, Statements, StoreFailure};

use boundary::{RecallRow, Score};

#[derive(Clone, Debug, Op)]
pub struct Recall {
    terms: BTreeSet<Term>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Recalled {
    node: NodeId,
    stage: Stage,
    title: Title,
    score: Score,
}

impl Recall {
    const MOST_TERMS: usize = 24;
    const MOST_NODES: usize = 4;
    const COMMON: [&'static str; 72] = [
        "a", "about", "after", "again", "all", "also", "an", "and", "any", "are", "as", "at", "be", "because", "been",
        "before", "being", "but", "by", "can", "could", "did", "do", "does", "doing", "done", "for", "from", "going",
        "had", "has", "have", "having", "how", "i", "if", "in", "into", "is", "it", "its", "just", "know", "let",
        "like", "make", "me", "more", "most", "much", "my", "need", "not", "of", "on", "only", "or", "our", "really",
        "should", "so", "some", "stuff", "supposed", "that", "the", "this", "to", "was", "we", "what", "with",
    ];

    #[must_use]
    pub fn telling(terms: &Terms) -> Recall {
        let common: BTreeSet<Term> = Self::COMMON.iter().map(|word| Term::new((*word).to_owned())).collect();
        let said: BTreeSet<Term> = [terms.first()].into_iter().chain(terms.rest()).cloned().collect();
        Recall { terms: said.difference(&common).take(Self::MOST_TERMS).cloned().collect() }
    }
}

impl Query for Recall {
    type Rows = Vec<Recalled>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<Recalled>, StoreFailure> {
        let rows = self
            .terms
            .iter()
            .map(|term| RecallRow::matching(term, statements))
            .collect::<Result<Vec<Vec<RecallRow>>, StoreFailure>>()?;
        let scored = rows.into_iter().flatten().fold(BTreeMap::new(), |mut nodes: BTreeMap<NodeId, Recalled>, row| {
            let recalled = row.recalled();
            let score = recalled.score;
            nodes.entry(recalled.node).and_modify(|held| held.raised(score)).or_insert(recalled);
            nodes
        });
        let mut ranked: Vec<Recalled> = scored.into_values().collect();
        ranked.sort_by(|left, right| right.score.ordered(left.score).then(left.node.cmp(&right.node)));
        Ok(ranked.into_iter().take(Self::MOST_NODES).collect())
    }
}

impl Recalled {
    fn raised(&mut self, score: Score) {
        self.score = self.score.plus(score);
    }
}

impl fmt::Display for Recalled {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "n{} {}: {}", self.node.into_inner(), self.stage, self.title)
    }
}

#[derive(Debug)]
pub struct Recollection<'recalled>(&'recalled [Recalled]);

impl<'recalled> From<&'recalled [Recalled]> for Recollection<'recalled> {
    fn from(recalled: &'recalled [Recalled]) -> Recollection<'recalled> {
        Recollection(recalled)
    }
}

impl fmt::Display for Recollection<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|recalled| writeln!(formatter, "{recalled}"))
    }
}
