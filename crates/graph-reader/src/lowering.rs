mod attributes;
mod boundary;
mod calls;
mod definition;
mod frames;
mod held;
mod laws;
mod places;
mod quotes;
mod regions;
mod seat;
mod signature;
mod uses;

use std::ffi::OsStr;
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use syn::spanned::Spanned;
use syn::{
    Attribute, Block, ExprCall, ExprMacro, ExprMethodCall, ExprPath, ForeignItemFn, Ident, ImplItemFn, ItemConst,
    ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType, ItemUnion, Macro,
    Signature as SynSignature, TraitItemFn,
};

use graph::{
    Alias, Assigned, Building, Entry, FileId, Function, Held, Implementation, Model, Mutation, Named, Origin, Output,
    Owner, Reach, Region, Signature, Sited, Spelling, Subject, TypeRef,
};
use lint_core::{SourceFile, Syntax};
use lint_layout::{Name, Position};

use crate::namespace::{
    Declared, DeclaredKind, Domain, Entries, Namespace, Openings, Resolver, Resolving, ScopeId, ScopeKind, Seeds,
    StdNames, Top, UsePath, Visibility,
};
use crate::parallel::{Parallel, Parked};
use crate::planting::Files;
use attributes::Attributes;
use boundary::SelfType;
use calls::CallRow;
use definition::{LoweredDefinition, Mutations};
use frames::{Enclosure, Frame};
use held::{Holder, HoldingRow, Link};
use laws::LawRow;
use lint_sweep::{Descend, Kind, Watch};
use places::{PlaceId, PlaceRows, Places};
use quotes::QuoteRow;
use regions::{Context, Regions};
use seat::{Built, Footing, Located, Seat};
pub use signature::ReceiverSyntax;
use signature::{DeclaredParameter, InScope, LoweredInput, LoweredParameter, Written};
use uses::UseRow;

#[derive(Debug)]
struct ImplRow {
    located: Located,
    subject: Name,
    ty: TypeRef<Written>,
    implemented: Option<UsePath>,
    domain: Domain,
    origin: Origin,
    region: Region,
    parameters: Vec<LoweredParameter>,
    assigned: Vec<LoweredAssigned>,
}

#[derive(Debug)]
struct LoweredAssigned {
    name: Name,
    ty: TypeRef<Written>,
}

impl ImplRow {
    fn written(seat: &Seat, region: Region, block: &ItemImpl, scope: &InScope) -> ImplRow {
        let ty = scope.lowered(&block.self_ty);
        ImplRow {
            located: seat.located(Position::from(block.impl_token.span)),
            subject: SelfType::written(&block.self_ty),
            ty,
            implemented: block.trait_.as_ref().map(|(_, path, _)| UsePath::from(path)),
            domain: Domain::Types,
            origin: Origin::Written,
            region,
            parameters: LoweredParameter::declared(scope, &block.generics),
            assigned: block
                .items
                .iter()
                .filter_map(Syntax::impl_type)
                .map(|associated| LoweredAssigned {
                    name: Name::from(&associated.ident),
                    ty: scope.lowered(&associated.ty),
                })
                .collect(),
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
            parameters: Vec::new(),
            assigned: Vec::new(),
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
            Implementation::new(subject, implemented, self.origin, self.region.joined(footing.outer())).parameterised(
                self.parameters.into_iter().map(|parameter| parameter.resolved(&resolving, located.scope())).collect(),
                self.assigned
                    .iter()
                    .map(|assigned| {
                        Assigned::new(assigned.name.clone(), Written::typed(&assigned.ty, &resolving, located.scope()))
                    })
                    .collect(),
            );
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
    parameters: Vec<LoweredParameter>,
    mutation: Mutation,
    definition: Option<LoweredDefinition>,
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
        let parameters = self.parameters.into_iter().map(|parameter| parameter.resolved(resolving, scope)).collect();
        let signature = Signature::new(inputs, Self::resolved(resolving, scope, self.output), constraints)
            .generic(parameters, self.mutation);
        let function = Function::new(self.name, self.owner, self.entry, signature)
            .seated(footing.places().seat(self.region.joined(footing.outer()), self.place))
            .defined(self.definition.map(|definition| definition.resolved(resolving, scope)));
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
    laws: Vec<LawRow>,
    quotes: Vec<QuoteRow>,
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
            laws: Footed::appended(columns.laws, footing, self.laws),
            quotes: Footed::appended(columns.quotes, footing, self.quotes),
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

impl Lowering {
    pub(crate) fn of_file(source: &SourceFile, reach: Reach) -> Lowering {
        let syntax = source.syntax();
        let shape = FileShape::from(source);
        let lowered = Lowered {
            openings: Openings::new(Entries::of_items(&syntax.items)),
            slots: Vec::new(),
            rows: FileRows::default(),
        };
        Lowering {
            lowered,
            here: Seat::new(ScopeId::FIRST, shape.children),
            enclosure: shape.enclosure,
            generics: Arc::new(InScope::outside()),
            context: Context::outermost(Regions::of_file(source, reach)),
            frames: Vec::new(),
        }
    }

    pub(crate) fn lowered(self) -> Lowered {
        self.lowered
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
    laws: Vec<Footed<'footing, LawRow>>,
    quotes: Vec<Footed<'footing, QuoteRow>>,
}

impl Columns<'_> {
    fn built(self, building: Building) -> Building {
        let with_aliases = Footed::every_built(self.aliases, building);
        let with_impls = Footed::every_built(self.impls, with_aliases);
        let with_functions = Footed::every_built(self.functions, with_impls);
        let with_holdings = Footed::every_built(self.holdings, with_functions);
        let with_calls = Footed::every_built(self.calls, with_holdings);
        let with_uses = Footed::every_built(self.uses, with_calls);
        let with_laws = Footed::every_built(self.laws, with_uses);
        Footed::every_built(self.quotes, with_laws)
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
            Parallel::available().mapped(&self.grown, |grown| grown.rows.places.settled(&resolving.based(grown.base)));
        let with_places = self
            .grown
            .iter()
            .zip(&places)
            .fold(with_exports, |built, (grown, seated)| seated.placed(grown.file, built));
        let mut footed: Vec<Footed<'_, FileRows>> = self
            .grown
            .into_iter()
            .zip(&places)
            .map(|(grown, seated)| Footed {
                footing: Footing::new(resolving.based(grown.base), grown.file, grown.outer, seated),
                row: grown.rows,
            })
            .collect();
        let aliases: Vec<Footed<'_, AliasRow>> = footed
            .iter_mut()
            .flat_map(|file| {
                let footing = file.footing;
                mem::take(&mut file.row.aliases).into_iter().map(move |row| Footed { footing, row })
            })
            .collect();
        let with_aliases = Footed::every_built(aliases, with_places);
        let parked: Vec<Parked<Footed<'_, FileRows>>> = footed.into_iter().map(Parked::from).collect();
        let fragments = Parallel::available().shares(&parked).worked(|share| {
            let files = share.iter().filter_map(Parked::taken);
            let columns = files.fold(Columns::default(), |columns, file| file.row.spread(file.footing, columns));
            columns.built(with_aliases.fragment())
        });
        fragments.into_iter().fold(with_aliases, Building::absorbed).built()
    }
}

#[derive(Debug)]
pub(crate) struct Lowering {
    lowered: Lowered,
    here: Seat,
    enclosure: Enclosure,
    generics: Arc<InScope>,
    context: Context,
    frames: Vec<Frame>,
}

impl Lowering {
    fn entered(&mut self, kind: ScopeKind, entries: Entries, children: Arc<Path>) -> Seat {
        let scope = self.lowered.openings.opened(self.here.scope(), kind, entries);
        let inner = Seat::new(scope, children);
        std::mem::replace(&mut self.here, inner)
    }

    fn module_entered(&mut self, item: &ItemMod, items: &[syn::Item]) -> Descend {
        let frame = self.frame();
        let name = Name::from(&item.ident);
        let children = Arc::from(self.here.children().join(&name));
        let outer = self.entered(ScopeKind::Module, Entries::of_items(items), children);
        let module = Declared::new(name, Visibility::from(&item.vis), DeclaredKind::Module(self.here.scope()));
        self.lowered.openings.declare(outer.scope(), module);
        self.enclosure = Enclosure::Module;
        self.context = self.context.within_module(item);
        self.saved(frame)
    }

    fn function(&mut self, attributes: &[Attribute], signature: &SynSignature, body: Option<&Block>) -> InScope {
        let generics = self.generics.extended(&signature.generics);
        let name = Name::from(&signature.ident);
        self.lowered.rows.functions.push(FnRow {
            located: self.here.located(Position::from(signature.ident.span())),
            owner: self.enclosure.owner(),
            entry: self.enclosure.entry(attributes, &name),
            inputs: generics.inputs(signature),
            output: generics.output(&signature.output),
            constraints: generics.constraints(&signature.generics),
            parameters: LoweredParameter::declared(&generics, &signature.generics),
            mutation: Mutations::signed(signature),
            definition: body.map(|block| LoweredDefinition::of_body(&generics, signature, block)),
            region: self.context.region(),
            place: self.context.place(),
            name,
        });
        generics
    }

    pub(super) fn method_entered(
        &mut self,
        attributes: &[Attribute],
        signature: &SynSignature,
        body: Option<&Block>,
    ) -> Descend {
        let frame = self.frame();
        self.context = self.context.within_item(attributes);
        let generics = self.function(attributes, signature, body);
        self.enclosed(Enclosure::Body, generics, frame)
    }

    fn enclosed(&mut self, enclosure: Enclosure, generics: InScope, frame: Frame) -> Descend {
        self.enclosure = enclosure;
        self.generics = Arc::new(generics);
        self.saved(frame)
    }
}

impl<'ast> Watch<'ast> for Lowering {
    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::ItemMod,
            Kind::Block,
            Kind::ItemImpl,
            Kind::ItemTrait,
            Kind::ItemFn,
            Kind::ImplItemFn,
            Kind::TraitItemFn,
            Kind::ForeignItemFn,
            Kind::ItemType,
            Kind::ItemStruct,
            Kind::ItemEnum,
            Kind::ItemUnion,
            Kind::ItemConst,
            Kind::ItemStatic,
            Kind::ExprMethodCall,
            Kind::ExprCall,
            Kind::ExprMacro,
            Kind::ExprPath,
            Kind::ItemMacro,
            Kind::Macro,
        ]
    }

    fn on_item_mod(&mut self, item: &'ast ItemMod) -> Descend {
        match &item.content {
            Some((_, items)) => self.module_entered(item, items),
            None => self.slotted(item),
        }
    }

    fn off_item_mod(&mut self, _: &'ast ItemMod) {
        self.restored();
    }

    fn on_block(&mut self, block: &'ast Block) -> Descend {
        let frame = self.frame();
        self.entered(ScopeKind::Block, Entries::of_block(&block.stmts), self.here.children());
        self.saved(frame)
    }

    fn off_block(&mut self, _: &'ast Block) {
        self.restored();
    }

    fn on_item_impl(&mut self, block: &'ast ItemImpl) -> Descend {
        let frame = self.frame();
        self.context = self.context.within_item(&block.attrs);
        self.impl_entered(block, frame)
    }

    fn off_item_impl(&mut self, _: &'ast ItemImpl) {
        self.restored();
    }

    fn on_item_trait(&mut self, item: &'ast ItemTrait) -> Descend {
        let frame = self.frame();
        let generics = self.generics.subjectless().extended(&item.generics);
        self.trait_bounds(item, &generics);
        let place = self.trait_owning(&item.ident, &item.attrs);
        self.filler_owning(item);
        self.context = self.context.owned(place);
        self.enclosed(Enclosure::Trait, generics, frame)
    }

    fn off_item_trait(&mut self, _: &'ast ItemTrait) {
        self.restored();
    }

    fn on_item_fn(&mut self, item: &'ast ItemFn) -> Descend {
        let frame = self.frame();
        let place = self.owning(&item.sig.ident, Domain::Values, &item.attrs);
        self.context = self.context.within_item(&item.attrs).owned(place);
        let generics = self.function(&item.attrs, &item.sig, Some(&item.block));
        self.enclosed(Enclosure::Body, generics, frame)
    }

    fn off_item_fn(&mut self, _: &'ast ItemFn) {
        self.restored();
    }

    fn on_impl_item_fn(&mut self, item: &'ast ImplItemFn) -> Descend {
        self.method_entered(&item.attrs, &item.sig, Some(&item.block))
    }

    fn off_impl_item_fn(&mut self, _: &'ast ImplItemFn) {
        self.restored();
    }

    fn on_trait_item_fn(&mut self, item: &'ast TraitItemFn) -> Descend {
        self.method_entered(&item.attrs, &item.sig, item.default.as_ref())
    }

    fn off_trait_item_fn(&mut self, _: &'ast TraitItemFn) {
        self.restored();
    }

    fn on_foreign_item_fn(&mut self, item: &'ast ForeignItemFn) -> Descend {
        let frame = self.frame();
        self.enclosure = Enclosure::Foreign;
        let generics = self.function(&item.attrs, &item.sig, None);
        self.enclosure = frame.enclosure();
        self.enclosed(Enclosure::Body, generics, frame)
    }

    fn off_foreign_item_fn(&mut self, _: &'ast ForeignItemFn) {
        self.restored();
    }

    fn on_item_type(&mut self, item: &'ast ItemType) -> Descend {
        let generics = InScope::outside().extended(&item.generics);
        let target = generics.lowered(&item.ty);
        self.hold(Holder::named(&item.ident, Held::Alias), Link::declaring(&item.ident), target.clone());
        self.lowered.rows.aliases.push(AliasRow {
            scope: self.here.scope(),
            name: UsePath::from(&item.ident),
            parameters: generics.declarations(&item.generics),
            target,
        });
        self.owned_by(&item.ident, Domain::Types, &item.attrs)
    }

    fn off_item_type(&mut self, _: &'ast ItemType) {
        self.restored();
    }

    fn on_item_struct(&mut self, item: &'ast ItemStruct) -> Descend {
        let scope = self.generics.extended(&item.generics);
        let link = Link::declaring(&item.ident);
        self.attributed(&Attributes::from(item.attrs.as_slice()), &item.ident);
        self.parameters(&Holder::named(&item.ident, Held::StructParameter), &item.generics, &link);
        self.struct_held(item, &scope, link);
        self.owned_by(&item.ident, Domain::Types, &item.attrs)
    }

    fn off_item_struct(&mut self, _: &'ast ItemStruct) {
        self.restored();
    }

    fn on_item_enum(&mut self, item: &'ast ItemEnum) -> Descend {
        let scope = self.generics.extended(&item.generics);
        let link = Link::declaring(&item.ident);
        self.attributed(&Attributes::from(item.attrs.as_slice()), &item.ident);
        self.parameters(&Holder::named(&item.ident, Held::Bound), &item.generics, &link);
        item.variants.iter().fold(&mut *self, |lowering, variant| {
            let holder = Holder::named(&variant.ident, Held::EnumField);
            lowering.fields_held(&holder, &link, variant.fields.iter(), &scope);
            lowering
        });
        self.owned_by(&item.ident, Domain::Types, &item.attrs)
    }

    fn off_item_enum(&mut self, _: &'ast ItemEnum) {
        self.restored();
    }

    fn on_item_union(&mut self, item: &'ast ItemUnion) -> Descend {
        let scope = self.generics.extended(&item.generics);
        let link = Link::declaring(&item.ident);
        self.attributed(&Attributes::from(item.attrs.as_slice()), &item.ident);
        self.fields_held(&Holder::named(&item.ident, Held::UnionField), &link, item.fields.named.iter(), &scope);
        self.owned_by(&item.ident, Domain::Types, &item.attrs)
    }

    fn off_item_union(&mut self, _: &'ast ItemUnion) {
        self.restored();
    }

    fn on_item_const(&mut self, item: &'ast ItemConst) -> Descend {
        self.valued(&item.ident, &item.ty, &item.attrs)
    }

    fn off_item_const(&mut self, _: &'ast ItemConst) {
        self.restored();
    }

    fn on_item_static(&mut self, item: &'ast ItemStatic) -> Descend {
        self.valued(&item.ident, &item.ty, &item.attrs)
    }

    fn off_item_static(&mut self, _: &'ast ItemStatic) {
        self.restored();
    }

    fn on_expr_method_call(&mut self, call: &'ast ExprMethodCall) -> Descend {
        self.method_called(call);
        Descend::Into
    }

    fn on_expr_call(&mut self, call: &'ast ExprCall) -> Descend {
        self.path_called(call);
        Descend::Into
    }

    fn on_expr_macro(&mut self, expr: &'ast ExprMacro) -> Descend {
        self.macro_called(expr);
        Descend::Into
    }

    fn on_expr_path(&mut self, expression: &'ast ExprPath) -> Descend {
        self.path_used(expression);
        Descend::Into
    }

    fn on_item_macro(&mut self, item: &'ast ItemMacro) -> Descend {
        match &item.ident {
            Some(ident) => self.owned_by(ident, Domain::Macros, &item.attrs),
            None => self.saved(self.frame()),
        }
    }

    fn off_item_macro(&mut self, item: &'ast ItemMacro) {
        self.macro_left(item).restored();
    }

    fn on_macro(&mut self, invoked: &'ast Macro) -> Descend {
        self.macro_used(invoked);
        Descend::Into
    }
}
