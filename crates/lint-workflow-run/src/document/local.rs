use crate::actions::Actions;
use crate::document::found::Found;
use crate::document::node::Node;
use crate::document::text::Text;
use crate::finding::Finding;

#[derive(Debug)]
pub(crate) struct Local;

impl Local {
    const PREFIX: &'static str = "./";

    pub(crate) fn finding(used: Node<'_, '_>, actions: Actions<'_>) -> Option<Found> {
        let target = used.text()?;
        let local = Text::new(target).after(Text::new(Self::PREFIX))?;
        (!actions.resolves(local.path()))
            .then(|| Found::new(used.line(), Finding::LocalAction { target: target.to_owned() }))
    }
}
