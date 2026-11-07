mod boundary;
mod command;
mod environment;
mod found;
mod key;
mod lines;
mod local;
mod node;
mod script;
mod text;

use saphyr::{MarkedYaml, YamlData};

use crate::actions::Actions;
use crate::finding::Finding;
use command::Command;
use environment::Environment;
use found::Found;
use key::Key;
use local::Local;
use node::Node;
use script::Script;

#[derive(Debug)]
pub(crate) struct Document<'text>(Vec<MarkedYaml<'text>>);

impl Document<'_> {
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
            Some(Key::Uses) => {
                Local::finding(value, actions).map(|finding| Found::new(value.line(), finding)).into_iter().collect()
            }
            Some(Key::Env) => Environment::findings(value).into_iter().chain(Self::judged(value, actions)).collect(),
            Some(Key::With) => Script::script(step, value)
                .map(|script| Found::new(script.line(), Finding::Script))
                .into_iter()
                .chain(Self::judged(value, actions))
                .collect(),
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
