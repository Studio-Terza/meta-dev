use crate::document::key::Key;
use crate::document::node::Node;
use crate::document::text::Text;

#[derive(Debug)]
pub(crate) struct Script;

impl Script {
    const ACTION: &'static str = "actions/github-script";

    pub(crate) fn script<'node, 'input>(step: Node<'_, '_>, with: Node<'node, 'input>) -> Option<Node<'node, 'input>> {
        step.entry(Key::Uses)
            .and_then(Node::text)
            .filter(|used| used.after(Text::new(Self::ACTION)).is_some())
            .and_then(|_| with.entry(Key::Script))
    }
}
