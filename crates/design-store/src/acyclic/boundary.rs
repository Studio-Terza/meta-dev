use std::marker::PhantomData;

use design_model::{NodeId, Ordinal, Segment};
use newtype::{Arbitrary, Counter, Newtype, Wrapping};
use store_ports::{Columns, Statements, StoreFailure, Written};

use super::{Acyclic, Closed, Refuses};
use transcript_ports::Uuid;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Keyed(i64);

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
struct Depth(i64);

#[derive(Debug, Columns)]
struct KeyRow {
    key: Keyed,
}

#[derive(Debug, Columns)]
struct Reached {
    depth: Depth,
}

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
        let message = AsRef::<Uuid>::as_ref(segment).clone();
        let ordinal = Ordinal::from(segment);
        let row = statements.one(store_ports::query!(
            KeyRow,
            r#"
SELECT
  rowid AS "key!: i64"
FROM
  segments
WHERE
  message = ?1
  AND ordinal = ?2
"#,
            message: Uuid,
            ordinal: Ordinal
        ))?;
        Ok(row.key)
    }
}

impl<R: Acyclic> Closing<R> {
    pub(crate) fn new(source: R::End, target: R::End) -> Closing<R> {
        Closing { source, target, relation: PhantomData }
    }

    pub(crate) fn written<Failure, Refused>(
        self,
        statements: &mut Statements<'_>,
        edge: impl FnOnce(&mut Statements<'_>) -> Result<Written, Failure>,
    ) -> Result<Written, Refused>
    where
        Refused: Refuses + From<Failure>,
    {
        let open = self.opened::<Refused>(statements)?;
        let edged = edge(statements)?;
        let closed = open.closed(R::CLOSED, statements)?;
        Ok(edged.plus(closed))
    }

    fn opened<Refused: Refuses>(self, statements: &mut Statements<'_>) -> Result<Open, Refused> {
        let open =
            Open { ancestor: R::keyed(&self.target, statements)?, descendant: R::keyed(&self.source, statements)? };
        let relation = R::CLOSED;
        let Open { ancestor, descendant } = &open;
        let reverse = statements.rows(store_ports::query!(
            Reached,
            r#"
SELECT
  depth AS "depth!: i64"
FROM
  closure
WHERE
  relation = ?1
  AND ancestor = ?2
  AND descendant = ?3
UNION ALL
SELECT
  0
WHERE
  ?2 = ?3
"#,
            relation: Closed,
            descendant: Keyed,
            ancestor: Keyed
        ))?;
        reverse
            .into_iter()
            .map(|reached| reached.depth)
            .next()
            .map_or(Ok(open), |_depth| Err(Refused::cyclic(R::cycle(self.source, self.target))))
    }
}

impl Open {
    fn closed(self, relation: Closed, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        let Open { ancestor, descendant } = self;
        statements.written(store_ports::query!(
            r#"
INSERT INTO
  closure (relation, ancestor, descendant, depth)
SELECT
  ?1,
  above.ancestor,
  below.descendant,
  above.depth + below.depth + 1
FROM
  (
    SELECT
      ?2 AS ancestor,
      0 AS depth
    UNION ALL
    SELECT
      ancestor,
      depth
    FROM
      closure
    WHERE
      relation = ?1
      AND descendant = ?2
  ) AS above,
  (
    SELECT
      ?3 AS descendant,
      0 AS depth
    UNION ALL
    SELECT
      descendant,
      depth
    FROM
      closure
    WHERE
      relation = ?1
      AND ancestor = ?3
  ) AS below
WHERE
  true
ON CONFLICT
  (relation, ancestor, descendant)
DO UPDATE SET
  depth = excluded.depth
WHERE
  excluded.depth < closure.depth
"#,
            relation: Closed,
            ancestor: Keyed,
            descendant: Keyed
        ))
    }
}
