use std::marker::PhantomData;

use design_model::{NodeId, Segment};
use newtype::{Counter, Newtype};
use store_ports::{Statements, StoreFailure, Written};

use super::{Acyclic, Refusal};
use crate::directions::Columns;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Keyed(i64);

#[derive(Debug)]
pub(crate) struct Closing<R: Acyclic> {
    source: R::End,
    target: R::End,
    relation: PhantomData<R>,
}

#[derive(Debug)]
struct Open {
    ancestor: Keyed,
    descendant: Keyed,
}

impl Keyed {
    pub(crate) fn node(id: NodeId) -> Keyed {
        Keyed(id.into_inner())
    }

    pub(crate) fn segment(segment: &Segment, statements: &mut Statements<'_>) -> Result<Keyed, StoreFailure> {
        Columns::from(segment).rowid(statements).map(Keyed)
    }
}

impl<R: Acyclic> Closing<R> {
    pub(crate) fn new(source: R::End, target: R::End) -> Closing<R> {
        Closing { source, target, relation: PhantomData }
    }

    pub(crate) fn written<Failure>(
        self,
        statements: &mut Statements<'_>,
        edge: impl FnOnce(&mut Statements<'_>) -> Result<Written, Failure>,
    ) -> Result<Written, Refusal>
    where
        Refusal: From<Failure>,
    {
        let open = self.opened(statements)?;
        let edged = edge(statements)?;
        let closed = open.closed(R::CLOSED.name(), statements)?;
        Ok(edged.plus(closed))
    }

    fn opened(self, statements: &mut Statements<'_>) -> Result<Open, Refusal> {
        let open =
            Open { ancestor: R::keyed(&self.target, statements)?, descendant: R::keyed(&self.source, statements)? };
        let relation = R::CLOSED.name();
        let reverse = statements.rows(sqlx::query!(
            r#"SELECT depth AS "depth!: i64" FROM closure WHERE relation = ?1 AND ancestor = ?2 AND descendant = ?3 UNION ALL SELECT 0 WHERE ?2 = ?3"#,
            relation,
            open.descendant.0,
            open.ancestor.0
        ))?;
        reverse.first().map_or(Ok(open), |_reached| Err(Refusal::Cyclic(R::cycle(self.source, self.target))))
    }
}

impl Open {
    fn closed(self, relation: &str, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        statements.written(sqlx::query!(
            "INSERT INTO closure (relation, ancestor, descendant, depth) SELECT ?1, above.ancestor, below.descendant, above.depth + below.depth + 1 FROM (SELECT ?2 AS ancestor, 0 AS depth UNION ALL SELECT ancestor, depth FROM closure WHERE relation = ?1 AND descendant = ?2) AS above, (SELECT ?3 AS descendant, 0 AS depth UNION ALL SELECT descendant, depth FROM closure WHERE relation = ?1 AND ancestor = ?3) AS below WHERE true ON CONFLICT (relation, ancestor, descendant) DO UPDATE SET depth = excluded.depth WHERE excluded.depth < closure.depth",
            relation,
            self.ancestor.0,
            self.descendant.0
        ))
    }
}
