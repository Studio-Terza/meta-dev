mod boundary;

use std::path::{Path, PathBuf};
use std::rc::Rc;

use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Attribute, Block, File, Ident, Item, ItemEnum, ItemImpl, ItemMod, ItemStruct, ItemUnion, Token};

use code_model::{Building, FileId, Found, Implementation, Model, Origin};
use lint_layout::{LineNumber, Name};

use crate::namespace::{
    CrateForm, Declared, DeclaredKind, Domain, Entries, Namespace, Resolver, ScopeId, ScopeKind, Seeds, UsePath,
    Visibility,
};
use crate::planting::{Files, ReadFile};
use boundary::SelfType;

#[derive(Clone, Debug)]
struct Seat {
    file: FileId,
    scope: ScopeId,
    children: Rc<Path>,
}

#[derive(Debug)]
struct ImplRow {
    file: FileId,
    line: LineNumber,
    scope: ScopeId,
    subject: Name,
    implemented: Option<UsePath>,
    domain: Domain,
    origin: Origin,
}

impl ImplRow {
    fn written(seat: &Seat, block: &ItemImpl) -> ImplRow {
        ImplRow {
            file: seat.file,
            line: LineNumber::from(block.impl_token.span),
            scope: seat.scope,
            subject: SelfType::written(&block.self_ty),
            implemented: block.trait_.as_ref().map(|(_, path, _)| UsePath::from(path)),
            domain: Domain::Types,
            origin: Origin::Written,
        }
    }

    fn derived(seat: &Seat, path: &syn::Path, subject: &Ident) -> ImplRow {
        ImplRow {
            file: seat.file,
            line: LineNumber::from(path.span()),
            scope: seat.scope,
            subject: Name::from(subject),
            implemented: Some(UsePath::from(path)),
            domain: Domain::Macros,
            origin: Origin::Derived,
        }
    }

    fn built(self, resolver: &Resolver<'_, '_>, mut building: Building) -> Building {
        let implemented = self.implemented.map(|path| resolver.path_resolved(self.scope, &path, self.domain));
        building.implementation(
            self.file,
            Found::new(self.line, Implementation::new(self.subject, implemented, self.origin)),
        );
        building
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotForm {
    Conventional,
    Pathed,
}

impl From<&[Attribute]> for SlotForm {
    fn from(attributes: &[Attribute]) -> SlotForm {
        match attributes.iter().find(|attribute| attribute.path().is_ident(Slot::PATH)) {
            Some(_) => SlotForm::Pathed,
            None => SlotForm::Conventional,
        }
    }
}

impl SlotForm {
    fn chosen(self, only: Option<PathBuf>) -> Option<PathBuf> {
        match self {
            SlotForm::Conventional => only,
            SlotForm::Pathed => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Slot {
    scope: ScopeId,
    name: Name,
    children: Rc<Path>,
    form: SlotForm,
    visibility: Visibility,
}

impl Slot {
    const PATH: &'static str = "path";
    const EXTENSION: &'static str = "rs";
    const MODULE_FILE: &'static str = "mod.rs";

    pub(crate) fn found(&self, files: &Files<'_>) -> Option<PathBuf> {
        let named = self.children.join(&self.name);
        let flat = named.with_extension(Self::EXTENSION);
        let nested = named.join(Self::MODULE_FILE);
        let found_flat = files.present(&flat).map(|_| flat);
        let found_nested = files.present(&nested).map(|_| nested);
        self.form.chosen(found_flat.xor(found_nested))
    }

    fn new(seat: &Seat, item: &ItemMod) -> Slot {
        Slot {
            scope: seat.scope,
            name: Name::from(&item.ident),
            children: Rc::clone(&seat.children),
            form: SlotForm::from(item.attrs.as_slice()),
            visibility: Visibility::from(&item.vis),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Lowered<'read> {
    space: Namespace<'read>,
    rows: Vec<ImplRow>,
}

impl<'read> Lowered<'read> {
    pub(crate) fn rooted(&mut self, read: ReadFile<'_>, name: Option<&'read Name>, form: CrateForm) -> Vec<Slot> {
        let syntax = read.source().syntax();
        let scope = self.space.planted(form, name, Entries::of_items(&syntax.items));
        let children = Rc::from(read.source().path().parent().unwrap_or_else(|| Path::new("")));
        self.lowered(Seat { file: read.file(), scope, children }, syntax)
    }

    pub(crate) fn planted(&mut self, slot: Slot, read: ReadFile<'_>) -> Vec<Slot> {
        let syntax = read.source().syntax();
        let scope = self.space.opened(slot.scope, ScopeKind::Module, Entries::of_items(&syntax.items));
        let children = Rc::from(slot.children.join(&slot.name));
        self.space.declare(slot.scope, Declared::new(slot.name, slot.visibility, DeclaredKind::Module(scope)));
        self.lowered(Seat { file: read.file(), scope, children }, syntax)
    }

    pub(crate) fn opaque(&mut self, slot: Slot) {
        self.space.declare(slot.scope, Declared::new(slot.name, slot.visibility, DeclaredKind::OpaqueModule));
    }

    pub(crate) fn island(&mut self, read: ReadFile<'_>) {
        let slots = self.rooted(read, None, CrateForm::Island);
        slots.into_iter().fold(self, |lowered, slot| {
            lowered.opaque(slot);
            lowered
        });
    }

    pub(crate) fn built(self, building: Building) -> Model {
        let seeds = Seeds::from(&self.space);
        let settled = seeds.settled();
        let tables = seeds.tables(&settled);
        let resolver = Resolver::new(&self.space, &tables);
        self.rows.into_iter().fold(building, |building, row| row.built(&resolver, building)).built()
    }

    fn lowered(&mut self, seat: Seat, syntax: &File) -> Vec<Slot> {
        let mut lowering = Lowering { lowered: self, here: seat, slots: Vec::new() };
        lowering.visit_file(syntax);
        lowering.slots
    }
}

#[derive(Debug)]
struct Lowering<'low, 'read> {
    lowered: &'low mut Lowered<'read>,
    here: Seat,
    slots: Vec<Slot>,
}

impl Lowering<'_, '_> {
    const DERIVE: &'static str = "derive";

    fn entered(&mut self, kind: ScopeKind, entries: Entries, children: Rc<Path>) -> Seat {
        let scope = self.lowered.space.opened(self.here.scope, kind, entries);
        let inner = Seat { file: self.here.file, scope, children };
        std::mem::replace(&mut self.here, inner)
    }

    fn module_visited(&mut self, item: &ItemMod, items: &[Item]) {
        let name = Name::from(&item.ident);
        let children = Rc::from(self.here.children.join(&name));
        let outer = self.entered(ScopeKind::Module, Entries::of_items(items), children);
        let module = Declared::new(name, Visibility::from(&item.vis), DeclaredKind::Module(self.here.scope));
        self.lowered.space.declare(outer.scope, module);
        visit::visit_item_mod(self, item);
        self.here = outer;
    }

    fn derived(&mut self, attributes: &[Attribute], subject: &Ident) {
        let derived: Vec<ImplRow> = attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::DERIVE))
            .filter_map(|attribute| {
                attribute.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated).ok()
            })
            .flatten()
            .map(|path| ImplRow::derived(&self.here, &path, subject))
            .collect();
        self.lowered.rows.extend(derived);
    }
}

impl<'ast> Visit<'ast> for Lowering<'_, '_> {
    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        match &item.content {
            Some((_, items)) => self.module_visited(item, items),
            None => self.slots.push(Slot::new(&self.here, item)),
        }
    }

    fn visit_block(&mut self, block: &'ast Block) {
        let outer = self.entered(ScopeKind::Block, Entries::of_block(&block.stmts), Rc::clone(&self.here.children));
        visit::visit_block(self, block);
        self.here = outer;
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        self.lowered.rows.push(ImplRow::written(&self.here, block));
        visit::visit_item_impl(self, block);
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.derived(&item.attrs, &item.ident);
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.derived(&item.attrs, &item.ident);
        visit::visit_item_enum(self, item);
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        self.derived(&item.attrs, &item.ident);
        visit::visit_item_union(self, item);
    }
}
