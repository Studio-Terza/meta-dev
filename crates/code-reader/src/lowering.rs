mod attributes;
mod boundary;
mod calls;
mod held;
mod places;
mod regions;
mod seat;
mod signature;
mod uses;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, ExprCall, ExprMacro, ExprMethodCall, ExprPath, ForeignItemFn, Ident, ImplItemFn, ItemConst,
    ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType, ItemUnion, Macro,
    Signature as SynSignature, TraitItemFn,
};

use code_model::{
    Alias, Building, Entry, FileId, Function, Held, Implementation, Model, Named, Origin, Output, Owner, Reach, Region,
    Signature, Sited, Spelling, Subject, TypeRef,
};
use lint_core::{Macros, SourceFile};
use lint_layout::{Name, Position, TestMarks, Whole, Word, words};

use crate::namespace::{
    Declared, DeclaredKind, Domain, Entries, Namespace, Openings, Resolver, Resolving, ScopeId, ScopeKind, Seeds,
    StdNames, Top, UsePath, Visibility,
};
use crate::parallel::Parallel;
use crate::planting::Files;
use attributes::Attributes;
use boundary::SelfType;
use calls::CallRow;
use held::{Holder, HoldingRow, Link};
use places::{PlaceId, PlaceRows, Places};
use regions::{Context, Regions};
use seat::{Built, Footing, Located, Seat};
use signature::{DeclaredParameter, InScope, LoweredInput, Written};
use uses::UseRow;

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
    fn written(seat: &Seat, region: Region, block: &ItemImpl, ty: TypeRef<Written>) -> ImplRow {
        ImplRow {
            located: seat.located(Position::from(block.impl_token.span)),
            subject: SelfType::written(&block.self_ty),
            ty,
            implemented: block.trait_.as_ref().map(|(_, path, _)| UsePath::from(path)),
            domain: Domain::Types,
            origin: Origin::Written,
            region,
        }
    }

    fn derived(seat: &Seat, region: Region, path: &syn::Path, subject: &Ident) -> ImplRow {
        let position = Position::from(path.span());
        ImplRow {
            located: seat.located(position),
            subject: Name::from(subject),
            ty: TypeRef::Named(Named::new(Written::path(UsePath::from(subject), position), Vec::new())),
            implemented: Some(UsePath::from(path)),
            domain: Domain::Macros,
            origin: Origin::Derived,
            region,
        }
    }

    fn derives(seat: &Seat, region: Region, attributes: &Attributes, subject: &Ident) -> Vec<ImplRow> {
        attributes.derived().iter().map(|path| ImplRow::derived(seat, region, path, subject)).collect()
    }
}

impl Built for ImplRow {
    fn built(self, footing: Footing<'_>, building: Building) -> Building {
        let located = self.located;
        let resolving = footing.resolving();
        let implemented = self.implemented.map(|path| {
            let spelling = Spelling::Written(path.written(), located.position());
            Sited::new(resolving.path(located.scope(), &path, self.domain), spelling)
        });
        let subject = Subject::new(self.subject, Written::typed(&self.ty, &resolving, located.scope()));
        let implementation =
            Implementation::new(subject, implemented, self.origin, self.region.joined(footing.outer()));
        located.implementation(footing, building, implementation)
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
    place: Option<PlaceId>,
}

impl FnRow {
    fn resolved(resolving: &Resolving<'_, '_, '_>, scope: ScopeId, output: Option<TypeRef<Written>>) -> Output {
        match output {
            Some(ty) => Output::Written(Written::typed(&ty, resolving, scope)),
            None => Output::Implicit,
        }
    }
}

impl Built for FnRow {
    fn built(self, footing: Footing<'_>, building: Building) -> Building {
        let scope = self.located.scope();
        let resolving = &footing.resolving();
        let inputs = self.inputs.into_iter().map(|input| input.typed(resolving, scope)).collect();
        let constraints = self.constraints.iter().map(|ty| Written::typed(ty, resolving, scope)).collect();
        let signature = Signature::new(inputs, Self::resolved(resolving, scope, self.output), constraints);
        let function = Function::new(self.name, self.owner, self.entry, signature)
            .seated(footing.places().seat(self.region.joined(footing.outer()), self.place));
        self.located.function(footing, building, function)
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
    fn built(self, footing: Footing<'_>, mut building: Building) -> Building {
        let resolving = &footing.resolving();
        let declared = resolving.path(self.scope, &self.name, Domain::Types).crate_item();
        let parameters =
            self.parameters.into_iter().map(|parameter| parameter.resolved(resolving, self.scope)).collect();
        let alias = Alias::new(parameters, Written::typed(&self.target, resolving, self.scope));
        declared.into_iter().zip([alias]).for_each(|(item, aliased)| building.alias(item, aliased));
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
    children: Arc<Path>,
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

    fn placed(self, base: ScopeId, outer: Region) -> Slot {
        Slot { scope: self.scope.above(base), region: self.region.joined(outer), ..self }
    }
}

#[derive(Debug, Default)]
pub(crate) struct FileRows {
    impls: Vec<ImplRow>,
    functions: Vec<FnRow>,
    aliases: Vec<AliasRow>,
    holdings: Vec<HoldingRow>,
    calls: Vec<CallRow>,
    uses: Vec<UseRow>,
    places: PlaceRows,
}

impl FileRows {
    fn spread<'footing>(self, footing: Footing<'footing>, columns: Columns<'footing>) -> Columns<'footing> {
        Columns {
            aliases: Footed::appended(columns.aliases, footing, self.aliases),
            impls: Footed::appended(columns.impls, footing, self.impls),
            functions: Footed::appended(columns.functions, footing, self.functions),
            holdings: Footed::appended(columns.holdings, footing, self.holdings),
            calls: Footed::appended(columns.calls, footing, self.calls),
            uses: Footed::appended(columns.uses, footing, self.uses),
        }
    }
}

#[derive(Debug)]
struct Footed<'footing, Row> {
    footing: Footing<'footing>,
    row: Row,
}

impl<'footing, Row: Built> Footed<'footing, Row> {
    fn appended(mut column: Vec<Self>, footing: Footing<'footing>, rows: Vec<Row>) -> Vec<Self> {
        column.extend(rows.into_iter().map(|row| Footed { footing, row }));
        column
    }

    fn every_built(column: Vec<Self>, building: Building) -> Building {
        column.into_iter().fold(building, |built, footed| footed.row.built(footed.footing, built))
    }
}

#[derive(Debug)]
pub(crate) struct Lowered {
    openings: Openings,
    slots: Vec<Slot>,
    rows: FileRows,
}

#[derive(Debug)]
struct FileShape {
    enclosure: Enclosure,
    children: Arc<Path>,
}

impl From<&SourceFile> for FileShape {
    fn from(source: &SourceFile) -> FileShape {
        let path = source.head().path();
        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        let rooted = Some(path).filter(|candidate| *candidate == AsRef::<Path>::as_ref(source.head().crate_file()));
        let enclosure = rooted
            .and_then(|root| source.head().placement().executable_root(source.head().located().within(root)))
            .map_or_default(|_| Enclosure::BinaryRoot);
        let module = rooted
            .map_or(Some(path), |_| None)
            .filter(|candidate| candidate.file_name().is_none_or(|name| name != OsStr::new(Slot::MODULE_FILE)));
        let children = module.map_or_else(|| parent.to_path_buf(), |stem| stem.with_extension(""));
        FileShape { enclosure, children: Arc::from(children) }
    }
}

impl Lowered {
    pub(crate) fn of_file(source: &SourceFile, reach: Reach) -> Lowered {
        let syntax = source.syntax();
        let shape = FileShape::from(source);
        let mut lowered = Lowered {
            openings: Openings::new(Entries::of_items(&syntax.items)),
            slots: Vec::new(),
            rows: FileRows::default(),
        };
        let mut lowering = Lowering {
            lowered: &mut lowered,
            here: Seat::new(ScopeId::FIRST, shape.children),
            enclosure: shape.enclosure,
            generics: InScope::outside(),
            context: Context::outermost(Regions::of_file(source, reach)),
        };
        lowering.visit_file(syntax);
        lowered
    }
}

#[derive(Debug)]
struct Grown {
    file: FileId,
    base: ScopeId,
    outer: Region,
    rows: FileRows,
}

#[derive(Debug, Default)]
struct Columns<'footing> {
    aliases: Vec<Footed<'footing, AliasRow>>,
    impls: Vec<Footed<'footing, ImplRow>>,
    functions: Vec<Footed<'footing, FnRow>>,
    holdings: Vec<Footed<'footing, HoldingRow>>,
    calls: Vec<Footed<'footing, CallRow>>,
    uses: Vec<Footed<'footing, UseRow>>,
}

impl Columns<'_> {
    fn built(self, building: Building) -> Building {
        let with_aliases = Footed::every_built(self.aliases, building);
        let with_impls = Footed::every_built(self.impls, with_aliases);
        let with_functions = Footed::every_built(self.functions, with_impls);
        let with_holdings = Footed::every_built(self.holdings, with_functions);
        let with_calls = Footed::every_built(self.calls, with_holdings);
        Footed::every_built(self.uses, with_calls)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Plot<'read> {
    space: Namespace<'read>,
    grown: Vec<Grown>,
}

impl<'read> Plot<'read> {
    pub(crate) fn rooted(&mut self, top: Top<'read>, file: FileId, lowered: Lowered) -> Vec<Slot> {
        self.grown_at(top, file, Region::SHIPPED, lowered)
    }

    pub(crate) fn planted(&mut self, slot: Slot, file: FileId, lowered: Lowered) -> Vec<Slot> {
        let top = Top::Child { parent: slot.scope, name: slot.name, visibility: slot.visibility };
        self.grown_at(top, file, slot.region, lowered)
    }

    pub(crate) fn opaque(&mut self, slot: Slot) {
        self.space.declare(slot.scope, Declared::new(slot.name, slot.visibility, DeclaredKind::OpaqueModule));
    }

    fn grown_at(&mut self, top: Top<'read>, file: FileId, outer: Region, lowered: Lowered) -> Vec<Slot> {
        let base = self.space.placed(top, lowered.openings);
        self.grown.push(Grown { file, base, outer, rows: lowered.rows });
        lowered.slots.into_iter().map(|slot| slot.placed(base, outer)).collect()
    }

    pub(crate) fn built(self, mut building: Building) -> Model {
        let seeds = Seeds::from(&self.space);
        let settled = seeds.settled();
        let tables = seeds.tables(&settled);
        let ids = self.space.issued(&mut building);
        let resolving = Resolving::new(Resolver::new(&self.space, &tables), &ids);
        let exports = self.space.exports();
        let with_exports =
            exports.iter().fold(StdNames::aliased(building), |built, export| export.recorded(&resolving, built));
        let places: Vec<Places> =
            Parallel::default().mapped(&self.grown, |grown| grown.rows.places.settled(&resolving.based(grown.base)));
        let columns = self.grown.into_iter().zip(&places).fold(Columns::default(), |columns, (grown, seated)| {
            grown.rows.spread(Footing::new(resolving.based(grown.base), grown.file, grown.outer, seated), columns)
        });
        columns.built(with_exports).built()
    }
}

#[derive(Debug)]
struct Lowering<'low> {
    lowered: &'low mut Lowered,
    here: Seat,
    enclosure: Enclosure,
    generics: InScope,
    context: Context,
}

impl Lowering<'_> {
    fn entered(&mut self, kind: ScopeKind, entries: Entries, children: Arc<Path>) -> Seat {
        let scope = self.lowered.openings.opened(self.here.scope(), kind, entries);
        let inner = Seat::new(scope, children);
        std::mem::replace(&mut self.here, inner)
    }

    fn module_visited(&mut self, item: &ItemMod, items: &[syn::Item]) {
        let name = Name::from(&item.ident);
        let children = Arc::from(self.here.children().join(&name));
        let outer = self.entered(ScopeKind::Module, Entries::of_items(items), children);
        let module = Declared::new(name, Visibility::from(&item.vis), DeclaredKind::Module(self.here.scope()));
        self.lowered.openings.declare(outer.scope(), module);
        let enclosing = std::mem::replace(&mut self.enclosure, Enclosure::Module);
        let context = self.context.within_module(item);
        self.contextual(context, |lowering| visit::visit_item_mod(lowering, item));
        self.enclosure = enclosing;
        self.here = outer;
    }

    fn function(&mut self, attributes: &[Attribute], signature: &SynSignature) -> InScope {
        let generics = self.generics.extended(&signature.generics);
        let name = Name::from(&signature.ident);
        self.lowered.rows.functions.push(FnRow {
            located: self.here.located(Position::from(signature.ident.span())),
            owner: self.enclosure.owner(),
            entry: self.enclosure.entry(attributes, &name),
            inputs: generics.inputs(signature),
            output: generics.output(&signature.output),
            constraints: generics.constraints(&signature.generics),
            region: self.context.region(),
            place: self.context.place(),
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

impl<'ast> Visit<'ast> for Lowering<'_> {
    fn visit_item_mod(&mut self, item: &'ast ItemMod) {
        match &item.content {
            Some((_, items)) => self.module_visited(item, items),
            None => self.lowered.slots.push(Slot::new(&self.here, self.context.region(), item)),
        }
    }

    fn visit_block(&mut self, block: &'ast Block) {
        let outer = self.entered(ScopeKind::Block, Entries::of_block(&block.stmts), self.here.children());
        visit::visit_block(self, block);
        self.here = outer;
    }

    fn visit_item_impl(&mut self, block: &'ast ItemImpl) {
        let tested = self.context.within_item(&block.attrs);
        self.contextual(tested, |lowering| lowering.impl_visited(block));
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        let generics = self.generics.subjectless().extended(&item.generics);
        let place = self.trait_owning(&item.ident);
        let context = self.context.owned(place);
        self.contextual(context, |lowering| {
            lowering.enclosed(Enclosure::Trait, generics, |enclosed| visit::visit_item_trait(enclosed, item));
        });
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        let place = self.owning(&item.sig.ident, Domain::Values);
        let tested = self.context.within_item(&item.attrs).owned(place);
        self.contextual(tested, |lowering| {
            let generics = lowering.function(&item.attrs, &item.sig);
            lowering.enclosed(Enclosure::Body, generics, |enclosed| visit::visit_item_fn(enclosed, item));
        });
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        let tested = self.context.within_item(&item.attrs);
        self.contextual(tested, |lowering| {
            let generics = lowering.function(&item.attrs, &item.sig);
            lowering.enclosed(Enclosure::Body, generics, |enclosed| visit::visit_impl_item_fn(enclosed, item));
        });
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
        self.hold(Holder::named(&item.ident, Held::Alias), Link::declaring(&item.ident), target.clone());
        self.lowered.rows.aliases.push(AliasRow {
            scope: self.here.scope(),
            name: UsePath::from(&item.ident),
            parameters: generics.declarations(&item.generics),
            target,
        });
        self.owned_by(&item.ident, Domain::Types, |lowering| visit::visit_item_type(lowering, item));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        let scope = self.generics.extended(&item.generics);
        let link = Link::declaring(&item.ident);
        self.attributed(&Attributes::from(item.attrs.as_slice()), &item.ident);
        self.parameters(&Holder::named(&item.ident, Held::StructParameter), &item.generics, &link);
        self.struct_held(item, &scope, link);
        self.owned_by(&item.ident, Domain::Types, |lowering| visit::visit_item_struct(lowering, item));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        let scope = self.generics.extended(&item.generics);
        let link = Link::declaring(&item.ident);
        self.attributed(&Attributes::from(item.attrs.as_slice()), &item.ident);
        item.variants.iter().fold(&mut *self, |lowering, variant| {
            let holder = Holder::named(&variant.ident, Held::EnumField);
            lowering.fields_held(&holder, &link, variant.fields.iter(), &scope);
            lowering
        });
        self.owned_by(&item.ident, Domain::Types, |lowering| visit::visit_item_enum(lowering, item));
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        let scope = self.generics.extended(&item.generics);
        let link = Link::declaring(&item.ident);
        self.attributed(&Attributes::from(item.attrs.as_slice()), &item.ident);
        self.fields_held(&Holder::named(&item.ident, Held::UnionField), &link, item.fields.named.iter(), &scope);
        self.owned_by(&item.ident, Domain::Types, |lowering| visit::visit_item_union(lowering, item));
    }

    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        let target = self.generics.lowered(&item.ty);
        self.hold(Holder::named(&item.ident, Held::Value), Link::declaring(&item.ident).valued(), target);
        self.owned_by(&item.ident, Domain::Values, |lowering| visit::visit_item_const(lowering, item));
    }

    fn visit_item_static(&mut self, item: &'ast ItemStatic) {
        let target = self.generics.lowered(&item.ty);
        self.hold(Holder::named(&item.ident, Held::Value), Link::declaring(&item.ident).valued(), target);
        self.owned_by(&item.ident, Domain::Values, |lowering| visit::visit_item_static(lowering, item));
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
        self.macro_called(expr);
        visit::visit_expr_macro(self, expr);
    }

    fn visit_expr_path(&mut self, expression: &'ast ExprPath) {
        self.path_used(expression);
        visit::visit_expr_path(self, expression);
    }

    fn visit_item_macro(&mut self, item: &'ast ItemMacro) {
        visit::visit_item_macro(self, item);
        let expanded = item.ident.as_ref().map_or_else(|| Macros::items(&item.mac), |_| Vec::new());
        expanded.iter().fold(self, |lowering, declared| {
            lowering.visit_item(declared);
            lowering
        });
    }

    fn visit_macro(&mut self, invoked: &'ast Macro) {
        self.macro_used(invoked);
        visit::visit_macro(self, invoked);
    }
}
