use saphyr::YamlData;

use crate::document::found::Found;
use crate::document::node::Node;
use crate::document::text::Text;
use crate::finding::Finding;

#[derive(Debug)]
pub(crate) struct Environment;

impl Environment {
    pub(crate) fn findings(values: Node<'_, '_>) -> Vec<Found> {
        match values.structure() {
            YamlData::Mapping(entries) => entries
                .iter()
                .filter(|(_, value)| Self::holds_lines(Node::new(value)))
                .map(|(name, _)| Found::new(Node::new(name).line(), Finding::Environment))
                .collect(),
            YamlData::Tagged(_, inner) => Self::findings(Node::new(inner)),
            _ => Vec::new(),
        }
    }

    fn holds_lines(value: Node<'_, '_>) -> bool {
        value.text().is_some_and(|text| Text::new(text).trimmed().spans_lines())
    }
}
