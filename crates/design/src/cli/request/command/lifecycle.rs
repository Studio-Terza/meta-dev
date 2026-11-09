use std::io::Write;

use design_model::{Casting, Entered, NodeId, Provenance, Restager, Role, Transition};
use design_store::{Cast, Restaged};
use store_ports::Store;

use super::Printed;
use super::subject::Found;
use crate::Outcome;
use crate::cli::failure::Failure;
use crate::cli::request::command::rendering::Refs;
use crate::cli::request::{Shaped, Token, Valued};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Moving {
    Refine,
    Specify,
    Reopen,
    Drop,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Move {
    node: NodeId,
    transition: Transition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recast {
    node: NodeId,
    role: Role,
}

impl Moving {
    fn wanted(self) -> &'static [Valued] {
        match self {
            Moving::Refine | Moving::Reopen => &[],
            Moving::Specify => &[Valued::Artifact, Valued::Condition],
            Moving::Drop => &[Valued::Reason],
        }
    }

    fn transition(self, shaped: Shaped<'_, 1>) -> Result<Transition, Failure> {
        match self {
            Moving::Refine => Ok(Transition::Refine),
            Moving::Specify => Ok(Transition::Specify {
                artifact: Token::read(shaped.flagged(Valued::Artifact)?)?,
                condition: Token::read(shaped.flagged(Valued::Condition)?)?,
            }),
            Moving::Reopen => Ok(Transition::Reopen),
            Moving::Drop => Ok(Transition::Drop { reason: Token::read(shaped.flagged(Valued::Reason)?)? }),
        }
    }
}

impl Move {
    pub fn read(moving: Moving, rest: &[Token]) -> Result<Move, Failure> {
        let shaped = Token::shaped(rest, moving.wanted())?;
        let [node] = shaped.words();
        Ok(Move { node: node.parse()?, transition: moving.transition(shaped)? })
    }

    pub fn written(self, store: &impl Store, out: &mut impl Write) -> Result<Outcome, Failure> {
        let lock = store.lock()?;
        let node = Found::resolved(store, self.node)?;
        store.write(Restaged::new(node, self.transition, Restager::Stage))?;
        let shown = Found::find(store, self.node)?;
        lock.end()?;
        Printed::lines(out, &Refs::placed(store, shown.placed())?)?;
        Ok(Outcome::Moved(shown.placed().node().stage().stage()))
    }
}

impl Recast {
    pub fn read(rest: &[Token]) -> Result<Recast, Failure> {
        let [node, role] = Token::shaped(rest, &[])?.words();
        Ok(Recast { node: node.parse()?, role: role.parse().map_err(|_unknown| Failure::Usage)? })
    }

    pub fn written(self, store: &impl Store, out: &mut impl Write) -> Result<Outcome, Failure> {
        let lock = store.lock()?;
        let node = Found::resolved(store, self.node)?;
        store.write(Cast::new(node, Casting::new(self.role, Provenance::from(Entered::Role))))?;
        let shown = Found::find(store, self.node)?;
        lock.end()?;
        Printed::lines(out, &Refs::placed(store, shown.placed())?)?;
        Ok(Outcome::Recast(shown.placed().node().casting().role()))
    }
}
