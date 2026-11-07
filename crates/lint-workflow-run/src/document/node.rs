use lint_layout::LineNumber;
use saphyr::{MarkedYaml, Scalar, YamlData};

use crate::document::key::Key;
use crate::document::text::Text;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Node<'node, 'input>(&'node MarkedYaml<'input>);

impl<'node, 'input> Node<'node, 'input> {
    pub(crate) const fn new(node: &'node MarkedYaml<'input>) -> Self {
        Node(node)
    }

    pub(crate) const fn structure(self) -> &'node YamlData<'input, MarkedYaml<'input>> {
        &self.0.data
    }

    pub(crate) fn line(self) -> LineNumber {
        LineNumber::from(self.0.span.start.line())
    }

    pub(crate) fn text(self) -> Option<Text<'node>> {
        match self.structure() {
            YamlData::Value(Scalar::String(text)) | YamlData::Representation(text, _, _) => Some(Text::new(text)),
            YamlData::Tagged(_, inner) => Node(inner).text(),
            _ => None,
        }
    }

    pub(crate) fn entry(self, key: Key) -> Option<Self> {
        match self.structure() {
            YamlData::Mapping(entries) => entries
                .iter()
                .find(|(name, _)| Node(name).text().and_then(Key::named) == Some(key))
                .map(|(_, value)| Node(value)),
            _ => None,
        }
    }
}
