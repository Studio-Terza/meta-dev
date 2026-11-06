use crate::lifecycle::{Live, Settled};
use crate::{AnyNode, Artifact, Condition, Misstaged, Reason, Refining, Vague};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Transition {
    Refine,
    Specify { artifact: Artifact, condition: Condition },
    Reopen,
    Drop { reason: Reason },
}

impl Transition {
    pub fn applied(self, node: AnyNode) -> Result<AnyNode, Misstaged> {
        match self {
            Transition::Refine => node.narrowed::<Vague>().map(|vague| vague.refine().any()),
            Transition::Specify { artifact, condition } => {
                node.narrowed::<Refining>().map(|refining| refining.specify(artifact, condition).any())
            }
            Transition::Reopen => node.narrowed::<Settled>().map(|settled| settled.refine().any()),
            Transition::Drop { reason } => node.narrowed::<Live>().map(|live| live.drop(reason).any()),
        }
    }
}
