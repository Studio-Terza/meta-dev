use std::collections::BTreeMap;
use std::path::PathBuf;

use lint_layout::{LineNumber, Name};

use crate::boundary::{FileId, ItemId};
use crate::call::Call;
use crate::domain::Domain;
use crate::found::Found;
use crate::function::Function;
use crate::holding::Holding;
use crate::implementation::Implementation;
use crate::item::{Alias, Item};
use crate::item_path::ItemPath;
use crate::path_use::PathUse;
use crate::resolved::Resolved;

use super::{Aliases, Declarations, FileModel, Filed, Model};

#[derive(Debug, Default)]
pub struct Building {
    model: Model,
    items: Vec<Item>,
    aliases: Aliases,
    exports: BTreeMap<Domain, BTreeMap<ItemPath, Resolved>>,
}

impl Building {
    #[must_use]
    pub fn fragment(&self) -> Building {
        Building { aliases: self.aliases.clone(), ..Building::default() }
    }

    #[must_use]
    pub fn absorbed(self, fragment: Building) -> Building {
        let mut model = self.model;
        let found = fragment.model;
        model.implementations.extend(found.implementations);
        model.functions.extend(found.functions);
        model.holdings.extend(found.holdings);
        model.calls.extend(found.calls);
        model.uses.extend(found.uses);
        model.interpolated.extend(found.interpolated);
        model.placed.extend(found.placed);
        Building { model, ..self }
    }

    #[must_use]
    pub fn aliased(&self, path: &Resolved) -> Option<&Resolved> {
        let alias = self.aliases.aliasing(path)?;
        alias.target().named().map(AsRef::as_ref)
    }

    pub fn file(&mut self, path: PathBuf, krate: Name, root: PathBuf) -> FileId {
        let file = FileId::next(&self.model.files);
        self.model.files.push(FileModel::new(path, krate, root));
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

    pub fn path_use(&mut self, file: FileId, line: LineNumber, used: PathUse) {
        let rows = used.mention().interpolated().map_or(&mut self.model.uses, |_| &mut self.model.interpolated);
        rows.push(Filed::new(file, Found::new(line, used)));
    }

    pub fn call(&mut self, file: FileId, line: LineNumber, call: Call) {
        self.model.calls.push(Filed::new(file, Found::new(line, call)));
    }

    pub fn placed(&mut self, file: FileId, line: LineNumber, item: ItemId) {
        self.model.placed.push(Filed::new(file, Found::new(line, item)));
    }

    pub fn declared(&mut self, item: Item) -> ItemId {
        let id = ItemId::next(&self.items);
        self.items.push(item);
        id
    }

    pub fn alias(&mut self, item: ItemId, alias: Alias) {
        self.aliases.declare(item, alias);
    }

    pub fn export(&mut self, domain: Domain, path: ItemPath, resolved: &Resolved) {
        self.exports.entry(domain).or_default().extend(resolved.known().map(|known| (path, known.clone())));
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
            placed: Filed::sorted(model.placed),
            uses: Filed::sorted(model.uses),
            interpolated: Filed::sorted(model.interpolated),
            declarations: Declarations::new(self.items, self.aliases, self.exports),
            roots: model.roots,
        }
    }
}
