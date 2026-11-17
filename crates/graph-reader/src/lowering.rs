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

use lint_sweep::{Drawn, Fold, Visit};
use std::ffi::OsStr;
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use syntax::Spanned;
use syntax::{
    Attribute, Block, ExprCall, ExprMacro, ExprMethodCall, ExprPath, ForeignItemFn, Ident, ImplItemConst, ImplItemFn,
    ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType, ItemUnion,
    Macro, Signature as SynSignature, TraitItemFn, Visibility as SynVisibility,
};

use graph::{
    Alias, Assigned, Building, Entry, Evaluation, Exposure, FileId, Function, Implementation, Model, Mutation, Named,
    Origin, Output, Owner, Reach, Region, Signature, Sited, Spelling, Subject, TypeRef,
};
use lint_core::{SourceFile, Syntax};
use lint_layout::{Name, Position};

use crate::namespace::{
    Declared, DeclaredKind, Domain, Entries, Exposing, Namespace, Openings, Resolver, Resolving, ScopeId, ScopeKind,
    Seeds, StdNames, Top, UsePath, Visibility,
};
use crate::parallel::{Parallel, Parked};
use crate::planting::Files;
use attributes::Attributes;
use boundary::SelfType;
use calls::CallRow;
pub(crate) use definition::BodyGroup;
use definition::{LoweredDefinition, Mutations, RowIndex};
use frames::{Enclosure, Frame};
use held::HoldingRow;
use laws::LawRow;
use lint_sweep::{Address, AttachingGroups, Attachment, Facts, Kind};
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

    fn derived(seat: &Seat, region: Region, path: &syntax::Path, subject: &Ident) -> ImplRow {
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
    exposure: Exposure,
    evaluation: Evaluation,
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
    fn defined(&mut self, definition: LoweredDefinition) {
        self.definition = Some(definition);
    }

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
            .exposed(self.exposure)
            .evaluated(self.evaluation)
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
        let path = AsRef::<Path>::as_ref(source.head());
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

impl<'read> Lowering<'read> {
    fn of_file(origin: Lowerer<'read>) -> Lowering<'read> {
        let source = origin.source;
        let reach = origin.reach;
        let syntax = source.syntax();
        let shape = FileShape::from(source);
        let lowered = Lowered {
            openings: Openings::new(Entries::of_items(&syntax.items)),
            slots: Vec::new(),
            rows: FileRows::default(),
        };
        Lowering {
            source,
            origin,
            opening: Vec::new(),
            lowered,
            here: Seat::new(ScopeId::FIRST, shape.children),
            enclosure: shape.enclosure,
            generics: Arc::new(InScope::outside()),
            context: Context::outermost(Regions::of_file(source, reach)),
            frames: Vec::new(),
            calling: None,
            tables: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Lowerer<'read> {
    source: &'read SourceFile,
    reach: Reach,
}

impl<'read> Lowerer<'read> {
    pub(crate) fn of_file(source: &'read SourceFile, reach: Reach) -> Lowerer<'read> {
        Lowerer { source, reach }
    }

    pub(crate) fn lowered(&self, found: Option<Box<Lowering<'read>>>) -> Lowered {
        self.opened(found).lowered
    }

    fn opened(&self, found: Option<Box<Lowering<'read>>>) -> Box<Lowering<'read>> {
        found.unwrap_or_else(|| Box::new(Lowering::of_file(*self)))
    }

    fn macro_left(lowering: Box<Lowering<'read>>, item: &ItemMacro) -> Box<Lowering<'read>> {
        match &item.ident {
            Some(_) => lowering.transcribed(item),
            None => lowering.expanded(&item.mac),
        }
    }

    fn stepped(
        &self,
        found: Option<Box<Lowering<'read>>>,
        step: impl FnOnce(Box<Lowering<'read>>) -> Box<Lowering<'read>>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        Visit::descending(Some(step(self.opened(found))))
    }

    fn settled(
        &self,
        found: Option<Box<Lowering<'read>>>,
        step: impl FnOnce(Box<Lowering<'read>>) -> Box<Lowering<'read>>,
    ) -> Box<Lowering<'read>> {
        step(self.opened(found))
    }
}

impl<'read> AttachingGroups<BodyGroup> for Lowerer<'read> {
    const ATTACHED: &'static [Kind] = BodyGroup::KINDS;

    fn attaching(
        &self,
        found: Option<Box<Lowering<'read>>>,
    ) -> Drawn<Option<Box<Lowering<'read>>>, Vec<Attachment<BodyGroup>>> {
        let mut lowering = self.opened(found);
        let opening = mem::take(&mut lowering.opening);
        Drawn::new(Some(lowering), opening)
    }

    fn detached(&self, found: Option<Box<Lowering<'read>>>, group: BodyGroup) -> Option<Box<Lowering<'read>>> {
        Some(self.settled(found, |lowering| lowering.defined(group)))
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

#[derive(Clone, Copy, Debug)]
pub(super) struct Method<'item> {
    attributes: &'item [Attribute],
    visibility: &'item SynVisibility,
    signature: &'item SynSignature,
    body: Option<&'item Block>,
}

#[derive(Debug)]
pub(crate) struct Lowering<'read> {
    source: &'read SourceFile,
    origin: Lowerer<'read>,
    opening: Vec<Attachment<BodyGroup>>,
    lowered: Lowered,
    here: Seat,
    enclosure: Enclosure,
    generics: Arc<InScope>,
    context: Context,
    frames: Vec<Frame>,
    calling: Option<Address>,
    tables: Vec<Address>,
}

impl<'read> Lowering<'read> {
    pub(super) fn entered(
        mut self: Box<Self>,
        kind: ScopeKind,
        entries: Entries,
        children: Arc<Path>,
    ) -> Box<Lowering<'read>> {
        let scope = self.lowered.openings.opened(self.here.scope(), kind, entries);
        self.context = self.context.entered(kind);
        self.here = Seat::new(scope, children);
        self
    }

    fn module_entered(self: Box<Self>, item: &ItemMod, items: &[syntax::Item]) -> Visit<Box<Lowering<'read>>> {
        let name = Name::from(&item.ident);
        let children = Arc::from(self.here.children().join(&name));
        let outer = self.here.scope();
        let inner = self.entered(ScopeKind::Module, Entries::of_items(items), children);
        let module = Declared::new(name, Visibility::from(&item.vis), DeclaredKind::Module(inner.here.scope()));
        Visit::descending(
            inner.declared(outer, module).enclosing(Enclosure::Module).contexted(|context| context.within_module(item)),
        )
    }

    pub(super) fn declared(mut self: Box<Self>, scope: ScopeId, declared: Declared) -> Box<Lowering<'read>> {
        self.lowered.openings.declare(scope, declared);
        self
    }

    fn defined(mut self: Box<Self>, group: BodyGroup) -> Box<Lowering<'read>> {
        let row = group.row();
        let definition = group.lowered();
        row.slot_in(&mut self.lowered.rows.functions)
            .into_iter()
            .zip([definition])
            .for_each(|(function, lowered)| function.defined(lowered));
        self
    }

    fn function(
        mut self: Box<Self>,
        attributes: &[Attribute],
        visibility: &SynVisibility,
        signature: &SynSignature,
        body: Option<&Block>,
    ) -> Box<Lowering<'read>> {
        let generics = self.generics.extended(&signature.generics);
        let name = Name::from(&signature.ident);
        let row = RowIndex::from(self.lowered.rows.functions.as_slice());
        self.opening
            .extend(body.map(|block| Attachment::reaching(block, BodyGroup::opened(row, &generics, signature, block))));
        self.lowered.rows.functions.push(FnRow {
            located: self.here.located(Position::from(signature.ident.span())),
            exposure: Exposing::declared_by(visibility),
            evaluation: signature.constness.map_or(Evaluation::Runtime, |_| Evaluation::Constant),
            owner: self.enclosure.owner(),
            entry: self.enclosure.entry(attributes, &name),
            inputs: generics.inputs(signature),
            output: generics.output(&signature.output),
            constraints: generics.constraints(&signature.generics),
            parameters: LoweredParameter::declared(&generics, &signature.generics),
            mutation: Mutations::signed(signature),
            definition: None,
            region: self.context.region(),
            place: self.context.place(),
            name,
        });
        self
    }

    pub(super) fn method_entered(self: Box<Self>, method: Method<'_>) -> Box<Lowering<'read>> {
        let generics = self.generics.extended(&method.signature.generics);
        Frame::saved(self)
            .contexted(|context| context.within_item(method.attributes))
            .function(method.attributes, method.visibility, method.signature, method.body)
            .enclosed(Enclosure::Body, generics)
    }

    pub(super) fn item_mod_entered(self: Box<Self>, item: &ItemMod) -> Visit<Box<Lowering<'read>>> {
        match &item.content {
            Some((_, items)) => self.module_entered(item, items),
            None => self.slotted(item),
        }
    }

    pub(super) fn impl_scoped(self: Box<Self>, block: &ItemImpl) -> Box<Lowering<'read>> {
        self.contexted(|context| context.within_item(&block.attrs)).impl_entered(block)
    }

    pub(super) fn trait_entered(self: Box<Self>, item: &ItemTrait) -> Box<Lowering<'read>> {
        let generics = self.generics.subjectless().extended(&item.generics);
        self.trait_bounds(item, &generics)
            .owned_by(&item.ident, Domain::Types, &item.attrs)
            .filler_owning(item)
            .contexted(|context| context.within_item(&item.attrs))
            .enclosed(Enclosure::Trait, generics)
    }

    pub(super) fn fn_entered(self: Box<Self>, item: &ItemFn) -> Box<Lowering<'read>> {
        let generics = self.generics.extended(&item.sig.generics);
        self.owned_by(&item.sig.ident, Domain::Values, &item.attrs)
            .contexted(|context| context.within_item(&item.attrs))
            .function(&item.attrs, &item.vis, &item.sig, Some(&item.block))
            .enclosed(Enclosure::Body, generics)
    }

    pub(super) fn foreign_entered(self: Box<Self>, item: &ForeignItemFn) -> Box<Lowering<'read>> {
        self.enclosing(Enclosure::Foreign).function(&item.attrs, &item.vis, &item.sig, None)
    }

    pub(super) fn tabled(
        self: Box<Self>,
        at: Address,
        ident: &Ident,
        ty: &syntax::Type,
        attributes: &[Attribute],
    ) -> Box<Lowering<'read>> {
        self.tabling(at).valued(ident, ty, attributes)
    }

    pub(super) fn macro_owned(self: Box<Self>, item: &ItemMacro) -> Box<Lowering<'read>> {
        match &item.ident {
            Some(ident) => self.owned_by(ident, Domain::Macros, &item.attrs),
            None => self,
        }
    }

    pub(super) fn tabling(mut self: Box<Self>, at: Address) -> Box<Lowering<'read>> {
        self.tables.push(at);
        self
    }

    pub(super) fn untabled(mut self: Box<Self>) -> Box<Lowering<'read>> {
        self.tables.pop();
        self
    }
}

impl<'read> Fold for Lowerer<'read> {
    type Found = Option<Box<Lowering<'read>>>;

    fn kinds(&self) -> &'static [Kind] {
        &[
            Kind::Item,
            Kind::Type,
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
            Kind::ImplItemConst,
            Kind::ExprMethodCall,
            Kind::ExprCall,
            Kind::ExprMacro,
            Kind::ExprPath,
            Kind::ExprStruct,
            Kind::ExprMatch,
            Kind::ExprLet,
            Kind::Local,
            Kind::ItemMacro,
            Kind::Macro,
        ]
    }

    fn on_item(
        &self,
        found: Option<Box<Lowering<'read>>>,
        _: &syntax::Item,
        _: &Facts<'_, syntax::Item>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, Frame::saved)
    }

    fn off_item(
        &self,
        found: Option<Box<Lowering<'read>>>,
        _: &syntax::Item,
        _: &Facts<'_, syntax::Item>,
    ) -> Option<Box<Lowering<'read>>> {
        Some(self.settled(found, Lowering::restored))
    }

    fn on_type(
        &self,
        found: Option<Box<Lowering<'read>>>,
        ty: &syntax::Type,
        _: &Facts<'_, syntax::Type>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.types_used(ty))
    }

    fn on_item_mod(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ItemMod,
        _: &Facts<'_, ItemMod>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.opened(found).item_mod_entered(item).mapped(Some)
    }

    fn on_block(
        &self,
        found: Option<Box<Lowering<'read>>>,
        block: &Block,
        _: &Facts<'_, Block>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.block_entered(block))
    }

    fn off_block(
        &self,
        found: Option<Box<Lowering<'read>>>,
        _: &Block,
        _: &Facts<'_, Block>,
    ) -> Option<Box<Lowering<'read>>> {
        Some(self.settled(found, Lowering::restored))
    }

    fn on_item_impl(
        &self,
        found: Option<Box<Lowering<'read>>>,
        block: &ItemImpl,
        _: &Facts<'_, ItemImpl>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.impl_scoped(block))
    }

    fn on_item_trait(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ItemTrait,
        _: &Facts<'_, ItemTrait>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.trait_entered(item))
    }

    fn on_item_fn(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ItemFn,
        _: &Facts<'_, ItemFn>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.fn_entered(item))
    }

    fn on_impl_item_fn(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        let method =
            Method { attributes: &item.attrs, visibility: &item.vis, signature: &item.sig, body: Some(&item.block) };
        self.stepped(found, |lowering| lowering.method_entered(method))
    }

    fn off_impl_item_fn(
        &self,
        found: Option<Box<Lowering<'read>>>,
        _: &ImplItemFn,
        _: &Facts<'_, ImplItemFn>,
    ) -> Option<Box<Lowering<'read>>> {
        Some(self.settled(found, Lowering::restored))
    }

    fn on_trait_item_fn(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        let method = Method {
            attributes: &item.attrs,
            visibility: &SynVisibility::Inherited,
            signature: &item.sig,
            body: item.default.as_ref(),
        };
        self.stepped(found, |lowering| lowering.method_entered(method))
    }

    fn off_trait_item_fn(
        &self,
        found: Option<Box<Lowering<'read>>>,
        _: &TraitItemFn,
        _: &Facts<'_, TraitItemFn>,
    ) -> Option<Box<Lowering<'read>>> {
        Some(self.settled(found, Lowering::restored))
    }

    fn on_foreign_item_fn(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ForeignItemFn,
        _: &Facts<'_, ForeignItemFn>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.foreign_entered(item))
    }

    fn on_item_type(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ItemType,
        _: &Facts<'_, ItemType>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.aliased(item))
    }

    fn on_item_struct(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ItemStruct,
        _: &Facts<'_, ItemStruct>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.struct_entered(item))
    }

    fn on_item_enum(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ItemEnum,
        _: &Facts<'_, ItemEnum>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.enum_entered(item))
    }

    fn on_item_union(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ItemUnion,
        _: &Facts<'_, ItemUnion>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.union_entered(item))
    }

    fn on_item_const(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ItemConst,
        facts: &Facts<'_, ItemConst>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.tabled(facts.address(), &item.ident, &item.ty, &item.attrs))
    }

    fn on_item_static(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ItemStatic,
        facts: &Facts<'_, ItemStatic>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.tabled(facts.address(), &item.ident, &item.ty, &item.attrs))
    }

    fn off_item_const(
        &self,
        found: Option<Box<Lowering<'read>>>,
        _: &ItemConst,
        _: &Facts<'_, ItemConst>,
    ) -> Option<Box<Lowering<'read>>> {
        Some(self.settled(found, Lowering::untabled))
    }

    fn off_item_static(
        &self,
        found: Option<Box<Lowering<'read>>>,
        _: &ItemStatic,
        _: &Facts<'_, ItemStatic>,
    ) -> Option<Box<Lowering<'read>>> {
        Some(self.settled(found, Lowering::untabled))
    }

    fn on_impl_item_const(
        &self,
        found: Option<Box<Lowering<'read>>>,
        _: &ImplItemConst,
        facts: &Facts<'_, ImplItemConst>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.tabling(facts.address()))
    }

    fn off_impl_item_const(
        &self,
        found: Option<Box<Lowering<'read>>>,
        _: &ImplItemConst,
        _: &Facts<'_, ImplItemConst>,
    ) -> Option<Box<Lowering<'read>>> {
        Some(self.settled(found, Lowering::untabled))
    }

    fn on_expr_method_call(
        &self,
        found: Option<Box<Lowering<'read>>>,
        call: &ExprMethodCall,
        _: &Facts<'_, ExprMethodCall>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.method_called(call))
    }

    fn on_expr_call(
        &self,
        found: Option<Box<Lowering<'read>>>,
        call: &ExprCall,
        _: &Facts<'_, ExprCall>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.path_called(call))
    }

    fn on_expr_macro(
        &self,
        found: Option<Box<Lowering<'read>>>,
        expr: &ExprMacro,
        _: &Facts<'_, ExprMacro>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.macro_called(expr))
    }

    fn on_expr_struct(
        &self,
        found: Option<Box<Lowering<'read>>>,
        literal: &syntax::ExprStruct,
        _: &Facts<'_, syntax::ExprStruct>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.struct_used(literal))
    }

    fn on_expr_path(
        &self,
        found: Option<Box<Lowering<'read>>>,
        expression: &ExprPath,
        _: &Facts<'_, ExprPath>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.path_used(expression))
    }

    fn on_expr_match(
        &self,
        found: Option<Box<Lowering<'read>>>,
        expr: &syntax::ExprMatch,
        _: &Facts<'_, syntax::ExprMatch>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| expr.arms.iter().fold(lowering, |matched, arm| matched.pattern_used(&arm.pat)))
    }

    fn on_expr_let(
        &self,
        found: Option<Box<Lowering<'read>>>,
        expr: &syntax::ExprLet,
        _: &Facts<'_, syntax::ExprLet>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.pattern_used(&expr.pat))
    }

    fn on_local(
        &self,
        found: Option<Box<Lowering<'read>>>,
        local: &syntax::Local,
        _: &Facts<'_, syntax::Local>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.diverged(local))
    }

    fn on_item_macro(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ItemMacro,
        _: &Facts<'_, ItemMacro>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.macro_owned(item))
    }

    fn off_item_macro(
        &self,
        found: Option<Box<Lowering<'read>>>,
        item: &ItemMacro,
        _: &Facts<'_, ItemMacro>,
    ) -> Option<Box<Lowering<'read>>> {
        Some(Self::macro_left(self.opened(found), item))
    }

    fn on_macro(
        &self,
        found: Option<Box<Lowering<'read>>>,
        invoked: &Macro,
        _: &Facts<'_, Macro>,
    ) -> Visit<Option<Box<Lowering<'read>>>> {
        self.stepped(found, |lowering| lowering.macro_used(invoked))
    }
}
