use std::path::PathBuf;

use lint_layout::LineNumber;

use crate::boundary::{FileId, ItemId};
use crate::found::Found;
use crate::function::Function;
use crate::implementation::Implementation;
use crate::item::{Alias, Item};
use crate::item_path::ItemPath;
use crate::model::{Aliases, Declarations, FileModel, Filed, Model};

#[derive(Debug, Default)]
pub struct Building {
    files: Vec<FileModel>,
    implementations: Vec<Filed<Implementation>>,
    functions: Vec<Filed<Function>>,
    items: Vec<Item>,
    aliases: Aliases,
}

impl Building {
    pub fn file(&mut self, path: PathBuf) -> FileId {
        let file = FileId::next(&self.files);
        self.files.push(FileModel::from(path));
        file
    }

    pub fn implementation(&mut self, file: FileId, found: Found<Implementation>) {
        self.implementations.push(Filed::new(file, found));
    }

    pub fn function(&mut self, file: FileId, line: LineNumber, function: Function) {
        self.functions.push(Filed::new(file, Found::new(line, function)));
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
    pub fn built(mut self) -> Model {
        self.implementations.sort_by_key(Filed::file);
        self.functions.sort_by_key(Filed::file);
        Model::new(self.files, self.implementations, self.functions, Declarations::new(self.items, self.aliases))
    }
}
