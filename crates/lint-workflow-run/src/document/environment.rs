use saphyr::YamlData;

use crate::document::found::Found;
use crate::document::node::Node;
use crate::finding::Finding;

#[derive(Debug)]
pub(crate) struct Environment;

impl Environment {
    pub(crate) fn findings(values: Node<'_, '_>) -> Vec<Found> {
        match values.structure() {
            YamlData::Mapping(entries) => entries
                .iter()
                .filter_map(|(name, value)| Self::multiline_entry(Node::new(name), Node::new(value)))
                .collect(),
            YamlData::Tagged(_, inner) => Self::findings(Node::new(inner)),
            _ => Vec::new(),
        }
    }

    fn multiline_entry(name: Node<'_, '_>, value: Node<'_, '_>) -> Option<Found> {
        value.text().and_then(|text| text.trimmed().multiline()).map(|_| Found::new(name.line(), Finding::Environment))
    }
}
