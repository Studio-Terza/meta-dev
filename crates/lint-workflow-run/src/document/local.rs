use crate::actions::Actions;
use crate::document::node::Node;
use crate::excerpt::Excerpt;
use crate::finding::Finding;
use crate::text::Text;

#[derive(Debug)]
pub(crate) struct Local;

impl Local {
    const PREFIX: &'static str = "./";

    pub(crate) fn finding(used: Node<'_, '_>, actions: Actions<'_>) -> Option<Finding> {
        let target = used.text()?;
        let local = target.after(Text::new(Self::PREFIX))?;
        match actions.resolving(local.path()) {
            Some(_) => None,
            None => Some(Finding::LocalAction { target: Excerpt::from(target) }),
        }
    }
}
