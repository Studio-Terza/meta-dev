use newtype::Newtype;

use crate::{Artifact, Condition, NodeId, Reason, Stage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Vague;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Refining;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Terminal {
    artifact: Artifact,
    condition: Condition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dropped {
    reason: Reason,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnyStage {
    Vague(Vague),
    Refining(Refining),
    Terminal(Terminal),
    Dropped(Dropped),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Live {
    Vague(Vague),
    Refining(Refining),
    Terminal(Terminal),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Settled {
    Terminal(Terminal),
    Dropped(Dropped),
}

pub trait Staging {
    type Output;

    fn vague(self, vague: &Vague) -> Self::Output;

    fn refining(self, refining: &Refining) -> Self::Output;

    fn terminal(self, terminal: &Terminal) -> Self::Output;

    fn dropped(self, dropped: &Dropped) -> Self::Output;
}

pub trait Refinable {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("n{} is {stage}, and that move does not start from {stage}", node.into_inner())]
pub struct Misstaged {
    node: NodeId,
    stage: Stage,
}

impl Terminal {
    #[must_use]
    pub fn new(artifact: Artifact, condition: Condition) -> Terminal {
        Terminal { artifact, condition }
    }

    #[must_use]
    pub fn artifact(&self) -> &Artifact {
        &self.artifact
    }

    #[must_use]
    pub fn condition(&self) -> &Condition {
        &self.condition
    }
}

impl From<Reason> for Dropped {
    fn from(reason: Reason) -> Dropped {
        Dropped { reason }
    }
}

impl Dropped {
    #[must_use]
    pub fn reason(&self) -> &Reason {
        &self.reason
    }
}

impl AnyStage {
    #[must_use]
    pub fn stage(&self) -> Stage {
        match self {
            AnyStage::Vague(_) => Stage::Vague,
            AnyStage::Refining(_) => Stage::Refining,
            AnyStage::Terminal(_) => Stage::Terminal,
            AnyStage::Dropped(_) => Stage::Dropped,
        }
    }

    pub fn staged<S: Staging>(&self, staging: S) -> S::Output {
        match self {
            AnyStage::Vague(vague) => staging.vague(vague),
            AnyStage::Refining(refining) => staging.refining(refining),
            AnyStage::Terminal(terminal) => staging.terminal(terminal),
            AnyStage::Dropped(dropped) => staging.dropped(dropped),
        }
    }
}

impl Refinable for Vague {}

impl Refinable for Settled {}

impl Misstaged {
    pub(crate) fn new(node: NodeId, stage: Stage) -> Misstaged {
        Misstaged { node, stage }
    }

    #[must_use]
    pub fn node(self) -> NodeId {
        self.node
    }

    #[must_use]
    pub fn stage(self) -> Stage {
        self.stage
    }
}

impl From<Vague> for AnyStage {
    fn from(vague: Vague) -> AnyStage {
        AnyStage::Vague(vague)
    }
}

impl From<Refining> for AnyStage {
    fn from(refining: Refining) -> AnyStage {
        AnyStage::Refining(refining)
    }
}

impl From<Terminal> for AnyStage {
    fn from(terminal: Terminal) -> AnyStage {
        AnyStage::Terminal(terminal)
    }
}

impl From<Dropped> for AnyStage {
    fn from(dropped: Dropped) -> AnyStage {
        AnyStage::Dropped(dropped)
    }
}

impl TryFrom<AnyStage> for Vague {
    type Error = AnyStage;

    fn try_from(stage: AnyStage) -> Result<Vague, AnyStage> {
        match stage {
            AnyStage::Vague(vague) => Ok(vague),
            other @ (AnyStage::Refining(_) | AnyStage::Terminal(_) | AnyStage::Dropped(_)) => Err(other),
        }
    }
}

impl TryFrom<AnyStage> for Refining {
    type Error = AnyStage;

    fn try_from(stage: AnyStage) -> Result<Refining, AnyStage> {
        match stage {
            AnyStage::Refining(refining) => Ok(refining),
            other @ (AnyStage::Vague(_) | AnyStage::Terminal(_) | AnyStage::Dropped(_)) => Err(other),
        }
    }
}

impl TryFrom<AnyStage> for Settled {
    type Error = AnyStage;

    fn try_from(stage: AnyStage) -> Result<Settled, AnyStage> {
        match stage {
            AnyStage::Terminal(terminal) => Ok(Settled::Terminal(terminal)),
            AnyStage::Dropped(dropped) => Ok(Settled::Dropped(dropped)),
            other @ (AnyStage::Vague(_) | AnyStage::Refining(_)) => Err(other),
        }
    }
}

impl TryFrom<AnyStage> for Live {
    type Error = AnyStage;

    fn try_from(stage: AnyStage) -> Result<Live, AnyStage> {
        match stage {
            AnyStage::Vague(vague) => Ok(Live::Vague(vague)),
            AnyStage::Refining(refining) => Ok(Live::Refining(refining)),
            AnyStage::Terminal(terminal) => Ok(Live::Terminal(terminal)),
            other @ AnyStage::Dropped(_) => Err(other),
        }
    }
}
