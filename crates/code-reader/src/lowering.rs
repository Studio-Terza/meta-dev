mod boundary;
mod calls;
mod held;
mod regions;
mod seat;
mod signature;

use std::path::{Path, PathBuf};
use std::rc::Rc;

use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, ExprCall, ExprMacro, ExprMethodCall, File, ForeignItemFn, Ident, ImplItemFn, ItemEnum, ItemFn,
    ItemImpl, ItemMod, ItemStruct, ItemTrait, ItemType, ItemUnion, Signature as SynSignature, Token, TraitItemFn,
};

use code_model::{
    Alias, Building, Entry, Function, Held, Implementation, Model, Named, Origin, Output, Owner, Region,
    Seat as Seated, Signature, Sited, Spelling, Subject, TypeRef,
};
use lint_layout::{LineNumber, Name, TestMarks, Whole, Word, words};

use crate::namespace::{
    CrateForm, Declared, DeclaredKind, Domain, Entries, Namespace, Resolver, Resolving, ScopeId, ScopeKind, Seeds,
    StdNames, UsePath, Visibility,
};
use crate::planting::{Files, ReadFile};
use boundary::SelfType;
use calls::CallRow;
use held::{Holder, HoldingRow, Link};
use regions::{Context, Regions};
use seat::{Built, Located, Seat};
use signature::{DeclaredParameter, InScope, LoweredInput, Written};

#[derive(Clone, Copy, Debug, Default)]
enum Enclosure {
    BinaryRoot,
    #[default]
    Module,
    Body,
    Impl(Owner),
    Trait,
    Foreign,
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
            Enclosure::BinaryRoot | Enclosure::Module => Owner::Free,
            Enclosure::Body => Owner::Nested,
            Enclosure::Impl(owner) => owner,
            Enclosure::Trait => Owner::TraitDecl,
            Enclosure::Foreign => Owner::Foreign,
        }
    }

    fn entry(self, attributes: &[Attribute], name: &Name) -> Entry {
        TestMarks::mark_test(attributes).map(|_| Entry::Test).or_else(|| self.main(name)).unwrap_or(Entry::Called)
    }

    fn main(self, name: &Name) -> Option<Entry> {
        match self {
            Enclosure::BinaryRoot => name.matches::<Whole>(&Self::MAIN).map(|_| Entry::Main),
            Enclosure::Module | Enclosure::Body | Enclosure::Impl(_) | Enclosure::Trait | Enclosure::Foreign => None,
        }
    }
}

#[derive(Debug)]
struct ImplRow {
    located: Located,
    subject: Name,
    ty: TypeRef<Written>,
    implemented: Option<UsePath>,
    domain: Domain,
    origin: Origin,
    region: Region,
}

impl ImplRow {
    const DERIVE: &'static str = "derive";

    fn written(seat: &Seat, region: Region, block: &ItemImpl, ty: TypeRef<Written>) -> ImplRow {
        ImplRow {
            located: seat.located(LineNumber::from(block.impl_token.span)),
            subject: SelfType::written(&block.self_ty),
            ty,
            implemented: block.trait_.as_ref().map(|(_, path, _)| UsePath::from(path)),
            domain: Domain::Types,
            origin: Origin::Written,
            region,
        }
    }

    fn derived(seat: &Seat, region: Region, path: &syn::Path, subject: &Ident) -> ImplRow {
        let line = LineNumber::from(path.span());
        ImplRow {
            located: seat.located(line),
            subject: Name::from(subject),
            ty: TypeRef::Named(Named::new(Written::path(UsePath::from(subject), line), Vec::new())),
            implemented: Some(UsePath::from(path)),
            domain: Domain::Macros,
            origin: Origin::Derived,
            region,
        }
    }

    fn derives(seat: &Seat, region: Region, attributes: &[Attribute], subject: &Ident) -> Vec<ImplRow> {
        attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident(Self::DERIVE))
            .filter_map(|attribute| {
                attribute.parse_args_with(Punctuated::<syn::Path, Token![,]>::parse_terminated).ok()
            })
            .flatten()
            .map(|path| ImplRow::derived(seat, region, &path, subject))
            .collect()
    }
}

impl Built for ImplRow {
    fn built(self, resolving: &Resolving<'_, '_, '_>, building: Building) -> Building {
        let located = self.located;
        let implemented = self.implemented.map(|path| {
            let spelling = Spelling::Written(path.written(), located.line());
            Sited::new(resolving.path(located.scope(), &path, self.domain), spelling)
        });
        let subject = Subject::new(self.subject, Written::typed(self.ty, resolving, located.scope()));
        let implementation = Implementation::new(subject, implemented, self.origin, self.region);
        located.implementation(building, implementation)
    }
}

#[derive(Debug)]
struct FnRow {
    located: Located,
    name: Name,
    owner: Owner,
    entry: Entry,
    inputs: Vec<LoweredInput>,
    output: Option<TypeRef<Written>>,
    constraints: Vec<TypeRef<Written>>,
    region: Region,
    subject: Option<TypeRef<Written>>,
}

impl FnRow {
    fn resolved(resolving: &Resolving<'_, '_, '_>, scope: ScopeId, output: Option<TypeRef<Written>>) -> Output {
        match output {
            Some(ty) => Output::Written(Written::typed(ty, resolving, scope)),
            None => Output::Implicit,
        }
    }
}

impl Built for FnRow {
    fn built(self, resolving: &Resolving<'_, '_, '_>, building: Building) -> Building {
        let scope = self.located.scope();
        let inputs = self.inputs.into_iter().map(|input| input.typed(resolving, scope)).collect();
        let constraints = self.constraints.into_iter().map(|ty| Written::typed(ty, resolving, scope)).collect();
        let signature = Signature::new(inputs, Self::resolved(resolving, scope, self.output), constraints);
        let subject = self.subject.map(|subject| Written::typed(subject, resolving, scope));
        let function =
            Function::new(self.name, self.owner, self.entry, signature).seated(Seated::new(self.region, subject));
        self.located.function(building, function)
    }
}

#[derive(Debug)]
struct AliasRow {
    scope: ScopeId,
    name: UsePath,
    parameters: Vec<DeclaredParameter>,
    target: TypeRef<Written>,
}

impl Built for AliasRow {
    fn built(self, resolving: &Resolving<'_, '_, '_>, mut building: Building) -> Building {
        let declared = resolving.path(self.scope, &self.name, Domain::Types).crate_item();
        let parameters =
            self.parameters.into_iter().map(|parameter| parameter.resolved(resolving, self.scope)).collect();
        let alias = Alias::new(parameters, Written::typed(self.target, resolving, self.scope));
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
    region: Region,
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

    fn new(seat: &Seat, region: Region, item: &ItemMod) -> Slot {
        Slot {
            scope: seat.scope(),
            name: Name::from(&item.ident),
            children: seat.children(),
            form: SlotForm::from(item.attrs.as_slice()),
            visibility: Visibility::from(&item.vis),
            region: Regions::of_module(region, item),
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Lowered<'read> {
    space: Namespace<'read>,
    rows: Vec<ImplRow>,
    functions: Vec<FnRow>,
    aliases: Vec<AliasRow>,
    holdings: Vec<HoldingRow>,
    calls: Vec<CallRow>,
}

impl<'read> Lowered<'read> {
    pub(crate) fn rooted(&mut self, read: ReadFile<'_>, name: Option<&'read Name>, form: CrateForm) -> Vec<Slot> {
        let enclosure = read
            .source()
            .placement()
            .executable_root(read.source().located().within(read.source().path()))
            .map(|_| Enclosure::BinaryRoot)
            .unwrap_or_default();
        self.root_lowered(read, name, form, enclosure)
    }

    pub(crate) fn planted(&mut self, slot: Slot, read: ReadFile<'_>) -> Vec<Slot> {
        let syntax = read.source().syntax();
        let scope = self.space.opened(slot.scope, ScopeKind::Module, Entries::of_items(&syntax.items));
        let children = Rc::from(slot.children.join(&slot.name));
        let region = Regions::joined(slot.region, Regions::of_file(read));
        self.space.declare(slot.scope, Declared::new(slot.name, slot.visibility, DeclaredKind::Module(scope)));
        self.lowered(Seat::new(read.file(), scope, children), Context::outermost(region), Enclosure::Module, syntax)
    }

    pub(crate) fn opaque(&mut self, slot: Slot) {
        self.space.declare(slot.scope, Declared::new(slot.name, slot.visibility, DeclaredKind::OpaqueModule));
    }

    pub(crate) fn island(&mut self, read: ReadFile<'_>) {
        let slots = self.root_lowered(read, None, CrateForm::Island, Enclosure::Module);
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
        let building = Self::each_built(self.aliases, &resolving, StdNames::aliased(building));
        let building = Self::each_built(self.rows, &resolving, building);
        let building = Self::each_built(self.functions, &resolving, building);
        let building = Self::each_built(self.holdings, &resolving, building);
        Self::each_built(self.calls, &resolving, building).built()
    }

    fn each_built<Row: Built>(rows: Vec<Row>, resolving: &Resolving<'_, '_, '_>, building: Building) -> Building {
        rows.into_iter().fold(building, |building, row| row.built(resolving, building))
    }

    fn root_lowered(
        &mut self,
        read: ReadFile<'_>,
        name: Option<&'read Name>,
        form: CrateForm,
        enclosure: Enclosure,
    ) -> Vec<Slot> {
        let syntax = read.source().syntax();
        let scope = self.space.planted(form, name, Entries::of_items(&syntax.items));
        let children = Rc::from(read.source().path().parent().unwrap_or_else(|| Path::new("")));
        let context = Context::outermost(Regions::of_file(read));
        self.lowered(Seat::new(read.file(), scope, children), context, enclosure, syntax)
    }

    fn lowered(&mut self, seat: Seat, context: Context, enclosure: Enclosure, syntax: &File) -> Vec<Slot> {
        let mut lowering =
            Lowering { lowered: self, here: seat, enclosure, generics: InScope::outside(), slots: Vec::new(), context };
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
    context: Context,
}

impl Lowering<'_, '_> {
    fn entered(&mut self, kind: ScopeKind, entries: Entries, children: Rc<Path>) -> Seat {
        let scope = self.lowered.space.opened(self.here.scope(), kind, entries);
        let inner = self.here.inner(scope, children);
        std::mem::replace(&mut self.here, inner)
    }

    fn module_visited(&mut self, item: &ItemMod, items: &[syn::Item]) {
        let name = Name::from(&item.ident);
        let children = Rc::from(self.here.children().join(&name));
        let outer = self.entered(ScopeKind::Module, Entries::of_items(items), children);
        let module = Declared::new(name, Visibility::from(&item.vis), DeclaredKind::Module(self.here.scope()));
        self.lowered.space.declare(outer.scope(), module);
        let enclosing = std::mem::replace(&mut self.enclosure, Enclosure::Module);
        let context = self.context.within_module(item);
        self.contextual(context, |lowering| visit::visit_item_mod(lowering, item));
        self.enclosure = enclosing;
        self.here = outer;
    }

    fn function(&mut self, attributes: &[Attribute], signature: &SynSignature) -> InScope {
        let generics = self.generics.extended(&signature.generics);
        let name = Name::from(&signature.ident);
        self.lowered.functions.push(FnRow {
            located: self.here.located(LineNumber::from(signature.ident.span())),
            owner: self.enclosure.owner(),
            entry: self.enclosure.entry(attributes, &name),
            inputs: generics.inputs(signature),
            output: generics.output(&signature.output),
            constraints: generics.constraints(&signature.generics),
            region: self.context.region(),
            subject: self.context.subject(),
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
            None => self.slots.push(Slot::new(&self.here, self.context.region(), item)),
        }
    }

    fn visit_block(&mut self, block: &'ast Block) {
        let outer = self.entered(ScopeKind::Block, Entries::of_block(&block.stmts), self.here.children());
        visit::visit_block(self, block);
        self.here = outer;
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        let declared = self.generics.extended(&block.generics);
        let subject = declared.lowered(&block.self_ty);
        let generics = declared.subject(subject.clone());
        self.lowered.rows.push(ImplRow::written(&self.here, self.context.region(), block, subject.clone()));
        let context = self.context.of_impl(block, subject);
        self.contextual(context, |lowering| {
            lowering.enclosed(Enclosure::of_impl(block), generics, |lowering| visit::visit_item_impl(lowering, block));
        });
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        let generics = self.generics.subjectless().extended(&item.generics);
        let context = self.context.of_trait();
        self.contextual(context, |lowering| {
            lowering.enclosed(Enclosure::Trait, generics, |lowering| visit::visit_item_trait(lowering, item));
        });
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

    fn visit_foreign_item_fn(&mut self, item: &'ast ForeignItemFn) {
        let enclosing = std::mem::replace(&mut self.enclosure, Enclosure::Foreign);
        let generics = self.function(&item.attrs, &item.sig);
        self.enclosure = enclosing;
        self.enclosed(Enclosure::Body, generics, |lowering| visit::visit_foreign_item_fn(lowering, item));
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        let generics = InScope::outside().extended(&item.generics);
        let target = generics.lowered(&item.ty);
        self.hold(Holder::named(&item.ident, Held::Alias), Link::Declares(UsePath::from(&item.ident)), target.clone());
        self.lowered.aliases.push(AliasRow {
            scope: self.here.scope(),
            name: UsePath::from(&item.ident),
            parameters: generics.declarations(&item.generics),
            target,
        });
        visit::visit_item_type(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.derived(&item.attrs, &item.ident);
        let link = Link::Declares(UsePath::from(&item.ident));
        self.parameters(&Holder::named(&item.ident, Held::StructParameter), &item.generics, &link);
        self.struct_held(item, &self.generics.extended(&item.generics), link);
        visit::visit_item_struct(self, item);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.derived(&item.attrs, &item.ident);
        let scope = self.generics.extended(&item.generics);
        item.variants.iter().fold(&mut *self, |lowering, variant| {
            lowering.fields_held(&Holder::named(&variant.ident, Held::EnumField), variant.fields.iter(), &scope);
            lowering
        });
        visit::visit_item_enum(self, item);
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        self.derived(&item.attrs, &item.ident);
        let scope = self.generics.extended(&item.generics);
        self.fields_held(&Holder::named(&item.ident, Held::UnionField), item.fields.named.iter(), &scope);
        visit::visit_item_union(self, item);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        self.method_called(call);
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        self.path_called(call);
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_macro(&mut self, expr: &'ast ExprMacro) {
        self.macro_called(&expr.mac);
        visit::visit_expr_macro(self, expr);
    }
}
