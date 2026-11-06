use std::path::PathBuf;

use lint_layout::LineNumber;

use crate::boundary::{FileId, ItemId};
use crate::call::Call;
use crate::found::Found;
use crate::function::Function;
use crate::holding::Holding;
use crate::implementation::Implementation;
use crate::item::{Alias, Item};
use crate::item_path::ItemPath;

use super::{Aliases, Declarations, FileModel, Filed, Model};

#[derive(Debug, Default)]
pub struct Building {
    model: Model,
    items: Vec<Item>,
    aliases: Aliases,
}

impl Building {
    pub fn file(&mut self, path: PathBuf) -> FileId {
        let file = FileId::next(&self.model.files);
        self.model.files.push(FileModel::from(path));
        file
    }

    pub fn implementation(&mut self, file: FileId, found: Found<Implementation>) {
        self.model.implementations.push(Filed::new(file, found));
    }

    pub fn function(&mut self, file: FileId, line: LineNumber, function: Function) {
        self.model.functions.push(Filed::new(file, Found::new(line, function)));
    }

    pub fn holding(&mut self, file: FileId, line: LineNumber, holding: Holding) {
        self.model.holdings.push(Filed::new(file, Found::new(line, holding)));
    }

    pub fn call(&mut self, file: FileId, line: LineNumber, call: Call) {
        self.model.calls.push(Filed::new(file, Found::new(line, call)));
    }

    pub fn declared(&mut self, item: Item) -> ItemId {
        let id = ItemId::next(&self.items);
        self.items.push(item);
        id
    }

    pub fn alias(&mut self, item: ItemId, alias: Alias) {
        self.aliases.declare(item, alias);
    }

    pub fn std_alias(&mut self, path: ItemPath, alias: Alias) {
        self.aliases.declare_std(path, alias);
    }

    #[must_use]
    pub fn built(self) -> Model {
        let model = self.model;
        Model {
            files: model.files,
            implementations: Filed::sorted(model.implementations),
            functions: Filed::sorted(model.functions),
            holdings: model.holdings,
            calls: model.calls,
            declarations: Declarations::new(self.items, self.aliases),
        }
    }
}
