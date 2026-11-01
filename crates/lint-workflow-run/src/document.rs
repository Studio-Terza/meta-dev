mod command;
mod found;
mod key;
mod local;
mod node;
mod script;
mod text;

use saphyr::{LoadableYamlNode, MarkedYaml, ScanError, YamlData};

use crate::actions::Actions;
use command::Command;
use found::Found;
use key::Key;
use local::Local;
use node::Node;
use script::Script;

#[derive(Debug)]
pub(crate) struct Document<'text>(Vec<MarkedYaml<'text>>);

impl<'text> Document<'text> {
    pub(crate) fn parse(text: &'text str) -> Result<Document<'text>, ScanError> {
        MarkedYaml::load_from_str(text).map(Document)
    }

    pub(crate) fn findings(&self, actions: Actions<'_>) -> Vec<Found> {
        self.0.iter().flat_map(|root| Self::judged(Node::new(root), actions)).collect()
    }

    fn judged(node: Node<'_, '_>, actions: Actions<'_>) -> Vec<Found> {
        match node.structure() {
            YamlData::Mapping(entries) => entries
                .iter()
                .flat_map(|(key, value)| Self::entry(Node::new(key), Node::new(value), node, actions))
                .collect(),
            YamlData::Sequence(items) => items.iter().flat_map(|item| Self::judged(Node::new(item), actions)).collect(),
            YamlData::Tagged(_, inner) => Self::judged(Node::new(inner), actions),
            _ => Vec::new(),
        }
    }

    fn entry(key: Node<'_, '_>, value: Node<'_, '_>, step: Node<'_, '_>, actions: Actions<'_>) -> Vec<Found> {
        match key.text().and_then(Key::named) {
            Some(setting @ (Key::Run | Key::Shell)) => Self::command(key, value, setting, actions),
            Some(Key::Uses) => Local::finding(value, actions).into_iter().collect(),
            Some(Key::With) => Script::finding(step, value).into_iter().chain(Self::judged(value, actions)).collect(),
            _ => Self::judged(value, actions),
        }
    }

    fn command(key: Node<'_, '_>, value: Node<'_, '_>, setting: Key, actions: Actions<'_>) -> Vec<Found> {
        match value.text() {
            Some(text) => {
                Command::new(text).finding(setting).map(|finding| Found::new(key.line(), finding)).into_iter().collect()
            }
            None => Self::judged(value, actions),
        }
    }
}
