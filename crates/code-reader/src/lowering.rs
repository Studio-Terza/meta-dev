mod boundary;
mod signature;

use std::path::{Path, PathBuf};
use std::rc::Rc;

use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, File, Ident, ImplItemFn, Item, ItemEnum, ItemFn, ItemImpl, ItemMod, ItemStruct, ItemTrait,
    ItemType, ItemUnion, Signature as SynSignature, Token, TraitItemFn,
};

use code_model::{
    Alias, Building, Entry, FileId, Found, Function, Implementation, Model, Named, Origin, Output, Owner, Signature,
    Subject, TypeRef,
};
use lint_layout::{LineNumber, Name, TestMarks, Whole, Word, words};

use crate::namespace::{
    CrateForm, Declared, DeclaredKind, Domain, Entries, Namespace, Resolver, Resolving, ScopeId, ScopeKind, Seeds,
    StdNames, UsePath, Visibility,
};
use crate::planting::{Files, ReadFile};
use boundary::SelfType;
use signature::{InScope, Written};

#[derive(Clone, Copy, Debug)]
enum Enclosure {
    Root,
    Module,
    Body,
    Impl(Owner),
    Trait,
}

impl Enclosure {
    const MAIN: [Word<'static>; 1] = words!["main"];

    fn of_impl(block: &ItemImpl) -> Enclosure {
        match block.trait_ {
            Some(_) => Enclosure::Impl(Owner::TraitImpl),
            None => Enclosure::Impl(Owner::Inherent),
        }
    }

    fn owner(self) -> Owner {
        match self {
            Enclosure::Root | Enclosure::Module => Owner::Free,
            Enclosure::Body => Owner::Nested,
            Enclosure::Impl(owner) => owner,
            Enclosure::Trait => Owner::TraitDecl,
        }
    }

    fn entry(self, attributes: &[Attribute], name: &Name) -> Entry {
        TestMarks::mark_test(attributes).map(|_| Entry::Test).or_else(|| self.main(name)).unwrap_or(Entry::Called)
    }

    fn main(self, name: &Name) -> Option<Entry> {
        match self {
            Enclosure::Root => name.matches::<Whole>(&Self::MAIN).map(|_| Entry::Main),
            Enclosure::Module | Enclosure::Body | Enclosure::Impl(_) | Enclosure::Trait => None,
        }
    }
}

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
    ty: TypeRef<Written>,
    implemented: Option<UsePath>,
    domain: Domain,
    origin: Origin,
}

impl ImplRow {
    const DERIVE: &'static str = "derive";

    fn written(seat: &Seat, block: &ItemImpl, generics: &InScope) -> ImplRow {
        ImplRow {
            file: seat.file,
            line: LineNumber::from(block.impl_token.span),
            scope: seat.scope,
            subject: SelfType::written(&block.self_ty),
            ty: generics.lowered(&block.self_ty),
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
            ty: TypeRef::Named(Named::new(Written::Path(UsePath::from(subject)), Vec::new())),
            implemented: Some(UsePath::from(path)),
            domain: Domain::Macros,
            origin: Origin::Derived,
        }
    }

    fn derives(seat: &Seat, attributes: &[Attribute], subject: &Ident) -> Vec<ImplRow> {
        attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::DERIVE))
            .filter_map(|attribute| {
                attribute.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated).ok()
            })
            .flatten()
            .map(|path| ImplRow::derived(seat, &path, subject))
            .collect()
    }

    fn built(self, resolving: &Resolving<'_, '_, '_>, mut building: Building) -> Building {
        let implemented = self.implemented.map(|path| resolving.path(self.scope, &path, self.domain));
        let subject = Subject::new(self.subject, Written::typed(self.ty, resolving, self.scope));
        building
            .implementation(self.file, Found::new(self.line, Implementation::new(subject, implemented, self.origin)));
        building
    }
}

#[derive(Debug)]
struct FnRow {
    file: FileId,
    line: LineNumber,
    scope: ScopeId,
    name: Name,
    owner: Owner,
    entry: Entry,
    output: Option<TypeRef<Written>>,
}

impl FnRow {
    fn built(self, resolving: &Resolving<'_, '_, '_>, mut building: Building) -> Building {
        let output = Self::resolved(resolving, self.scope, self.output);
        let function = Function::new(self.name, self.owner, self.entry, Signature::from(output));
        building.function(self.file, self.line, function);
        building
    }

    fn resolved(resolving: &Resolving<'_, '_, '_>, scope: ScopeId, output: Option<TypeRef<Written>>) -> Output {
        match output {
            Some(ty) => Output::Written(Written::typed(ty, resolving, scope)),
            None => Output::Implicit,
        }
    }
}

#[derive(Debug)]
struct AliasRow {
    scope: ScopeId,
    name: UsePath,
    parameters: Vec<Name>,
    target: TypeRef<Written>,
}

impl AliasRow {
    fn built(self, resolving: &Resolving<'_, '_, '_>, mut building: Building) -> Building {
        let declared = resolving.path(self.scope, &self.name, Domain::Types).crate_item();
        let alias = Alias::new(self.parameters, Written::typed(self.target, resolving, self.scope));
        declared.into_iter().zip([alias]).for_each(|(item, alias)| building.alias(item, alias));
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
    functions: Vec<FnRow>,
    aliases: Vec<AliasRow>,
}

impl<'read> Lowered<'read> {
    pub(crate) fn rooted(&mut self, read: ReadFile<'_>, name: Option<&'read Name>, form: CrateForm) -> Vec<Slot> {
        let syntax = read.source().syntax();
        let scope = self.space.planted(form, name, Entries::of_items(&syntax.items));
        let children = Rc::from(read.source().path().parent().unwrap_or_else(|| Path::new("")));
        self.lowered(Seat { file: read.file(), scope, children }, Enclosure::Root, syntax)
    }

    pub(crate) fn planted(&mut self, slot: Slot, read: ReadFile<'_>) -> Vec<Slot> {
        let syntax = read.source().syntax();
        let scope = self.space.opened(slot.scope, ScopeKind::Module, Entries::of_items(&syntax.items));
        let children = Rc::from(slot.children.join(&slot.name));
        self.space.declare(slot.scope, Declared::new(slot.name, slot.visibility, DeclaredKind::Module(scope)));
        self.lowered(Seat { file: read.file(), scope, children }, Enclosure::Module, syntax)
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

    pub(crate) fn built(self, mut building: Building) -> Model {
        let seeds = Seeds::from(&self.space);
        let settled = seeds.settled();
        let tables = seeds.tables(&settled);
        let ids = self.space.issued(&mut building);
        let resolving = Resolving::new(Resolver::new(&self.space, &tables), &ids);
        let building =
            self.aliases.into_iter().fold(StdNames::aliased(building), |building, row| row.built(&resolving, building));
        let building = self.rows.into_iter().fold(building, |building, row| row.built(&resolving, building));
        self.functions.into_iter().fold(building, |building, row| row.built(&resolving, building)).built()
    }

    fn lowered(&mut self, seat: Seat, enclosure: Enclosure, syntax: &File) -> Vec<Slot> {
        let mut lowering =
            Lowering { lowered: self, here: seat, enclosure, generics: InScope::default(), slots: Vec::new() };
        lowering.visit_file(syntax);
        lowering.slots
    }
}

#[derive(Debug)]
struct Lowering<'low, 'read> {
    lowered: &'low mut Lowered<'read>,
    here: Seat,
    enclosure: Enclosure,
    generics: InScope,
    slots: Vec<Slot>,
}

impl Lowering<'_, '_> {
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
        let enclosing = std::mem::replace(&mut self.enclosure, Enclosure::Module);
        visit::visit_item_mod(self, item);
        self.enclosure = enclosing;
        self.here = outer;
    }

    fn function(&mut self, attributes: &[Attribute], signature: &SynSignature) -> InScope {
        let generics = self.generics.extended(&signature.generics);
        let name = Name::from(&signature.ident);
        self.lowered.functions.push(FnRow {
            file: self.here.file,
            line: LineNumber::from(signature.ident.span()),
            scope: self.here.scope,
            owner: self.enclosure.owner(),
            entry: self.enclosure.entry(attributes, &name),
            output: generics.output(&signature.output),
            name,
        });
        generics
    }

    fn enclosed(&mut self, enclosure: Enclosure, generics: InScope, visited: impl FnOnce(&mut Self)) {
        let enclosing = std::mem::replace(&mut self.enclosure, enclosure);
        let outer = std::mem::replace(&mut self.generics, generics);
        visited(self);
        self.generics = outer;
        self.enclosure = enclosing;
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
        let generics = self.generics.extended(&block.generics);
        self.lowered.rows.push(ImplRow::written(&self.here, block, &generics));
        self.enclosed(Enclosure::of_impl(block), generics, |lowering| visit::visit_item_impl(lowering, block));
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        let generics = self.generics.extended(&item.generics);
        self.enclosed(Enclosure::Trait, generics, |lowering| visit::visit_item_trait(lowering, item));
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        let generics = self.function(&item.attrs, &item.sig);
        self.enclosed(Enclosure::Body, generics, |lowering| visit::visit_item_fn(lowering, item));
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        let generics = self.function(&item.attrs, &item.sig);
        self.enclosed(Enclosure::Body, generics, |lowering| visit::visit_impl_item_fn(lowering, item));
    }

    fn visit_trait_item_fn(&mut self, item: &'ast TraitItemFn) {
        let generics = self.function(&item.attrs, &item.sig);
        self.enclosed(Enclosure::Body, generics, |lowering| visit::visit_trait_item_fn(lowering, item));
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        let generics = InScope::default().extended(&item.generics);
        self.lowered.aliases.push(AliasRow {
            scope: self.here.scope,
            name: UsePath::from(&item.ident),
            parameters: generics.parameters(),
            target: generics.lowered(&item.ty),
        });
        visit::visit_item_type(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.lowered.rows.extend(ImplRow::derives(&self.here, &item.attrs, &item.ident));
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.lowered.rows.extend(ImplRow::derives(&self.here, &item.attrs, &item.ident));
        visit::visit_item_enum(self, item);
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        self.lowered.rows.extend(ImplRow::derives(&self.here, &item.attrs, &item.ident));
        visit::visit_item_union(self, item);
    }
}
