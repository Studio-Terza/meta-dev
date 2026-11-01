use crate::document::found::Found;
use crate::document::key::Key;
use crate::document::node::Node;
use crate::document::text::Text;
use crate::finding::Finding;

#[derive(Debug)]
pub(crate) struct Script;

impl Script {
    const ACTION: &'static str = "actions/github-script";

    pub(crate) fn finding(step: Node<'_, '_>, with: Node<'_, '_>) -> Option<Found> {
        step.entry(Key::Uses)
            .and_then(Node::text)
            .filter(|used| Text::new(used).after(Text::new(Self::ACTION)).is_some())
            .and_then(|_| with.entry(Key::Script))
            .map(|script| Found::new(script.line(), Finding::Script))
    }
}
