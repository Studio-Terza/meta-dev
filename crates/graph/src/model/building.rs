use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use lint_layout::LineNumber;

use crate::boundary::{FileId, ItemId};
use crate::call::Call;
use crate::dispatch::Dispatch;
use crate::domain::Domain;
use crate::found::Found;
use crate::function::Function;
use crate::holding::Holding;
use crate::implementation::Implementation;
use crate::item::{Alias, Item};
use crate::item_declaration::ItemDeclaration;
use crate::item_path::ItemPath;
use crate::law::Law;
use crate::path_use::{Mention, PathUse};
use crate::placed::FileHome;
use crate::quoted::Quoted;
use crate::resolved::Resolved;

use super::{Aliases, Declarations, FileModel, Filed, Model};

#[derive(Debug, Default)]
pub struct Building {
    model: Model,
    items: Vec<Item>,
    aliases: Aliases,
    exports: BTreeMap<Domain, BTreeMap<ItemPath, Resolved>>,
}

impl Model {
    fn filed(&mut self, file: FileId, line: LineNumber, used: PathUse) {
        match used.mention() {
            Mention::Interpolated => self.interpolated.push(Filed::new(file, Found::new(line, used))),
            Mention::Typed => self.typed.push(Filed::new(file, Found::new(line, used))),
            Mention::Dispatched => self.dispatched.push(Dispatch::from(&used)),
            Mention::Expression | Mention::Invoked | Mention::InMacro | Mention::Matched | Mention::Expanded => {
                self.uses.push(Filed::new(file, Found::new(line, used)));
            }
        }
    }
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
        model.typed.extend(found.typed);
        model.dispatched.extend(found.dispatched);
        model.placed.extend(found.placed);
        model.laws.extend(found.laws);
        model.quoted.extend(found.quoted);
        Building { model, ..self }
    }

    #[must_use]
    pub fn aliased(&self, path: &Resolved) -> Option<&Resolved> {
        let alias = self.aliases.aliasing(path)?;
        alias.target().named().map(AsRef::as_ref)
    }

    pub fn file(&mut self, path: PathBuf, home: FileHome) -> FileId {
        let file = FileId::next(&self.model.files);
        self.model.files.push(FileModel::new(path, home));
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
        self.model.filed(file, line, used);
    }

    pub fn law(&mut self, file: FileId, line: LineNumber, law: Law) {
        self.model.laws.push(Filed::new(file, Found::new(line, law)));
    }

    pub fn quoted(&mut self, file: FileId, line: LineNumber, quoted: Quoted) {
        self.model.quoted.push(Filed::new(file, Found::new(line, quoted)));
    }

    pub fn call(&mut self, file: FileId, line: LineNumber, call: Call) {
        self.model.calls.push(Filed::new(file, Found::new(line, call)));
    }

    pub fn placed(&mut self, file: FileId, line: LineNumber, item: ItemDeclaration) {
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
            laws: Filed::sorted(model.laws),
            uses: Filed::sorted(model.uses),
            interpolated: Filed::sorted(model.interpolated),
            typed: Filed::sorted(model.typed),
            dispatched: model.dispatched,
            quoted: Filed::sorted(model.quoted),
            declarations: Declarations::new(self.items, self.aliases, self.exports),
            roots: model.roots,
            references: OnceLock::new(),
        }
    }
}
