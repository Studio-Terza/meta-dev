use graph::{Function, Named, Resolved, Sited, TypeUses};
use lint_layout::{Name, Word};

#[derive(Debug, Default)]
pub(super) struct TakesNode(Option<()>);

impl TypeUses for TakesNode {
    fn named(&mut self, named: &Named<Sited>) {
        let resolved: &Resolved = named.as_ref();
        let nodes = Name::from(TakesNode::NODES);
        let root = resolved.external().and_then(|external| external.path().as_ref().first());
        self.0 = self.0.or(root.filter(|found| **found == nodes).map(|_| ()));
        named.each_inner_use(self);
    }
}

impl TakesNode {
    const NODES: Word<'static> = Word::new("syn");

    pub(super) fn function(function: &Function) -> Option<()> {
        let typed = function.signature().inputs().iter().filter_map(|input| input.typed());
        typed
            .fold(TakesNode::default(), |mut found, ty| {
                ty.each_use(&mut found);
                found
            })
            .0
    }
}
