use crate::actions::Actions;
use crate::document::node::Node;
use crate::document::text::Text;
use crate::excerpt::Excerpt;
use crate::finding::Finding;

#[derive(Debug)]
pub(crate) struct Local;

impl Local {
    const PREFIX: &'static str = "./";

    pub(crate) fn finding(used: Node<'_, '_>, actions: Actions<'_>) -> Option<Finding> {
        let target = used.text()?;
        let local = target.after(Text::new(Self::PREFIX))?;
        actions.resolving(local.path()).map_or(Some(Finding::LocalAction { target: Excerpt::from(target) }), |_| None)
    }
}
