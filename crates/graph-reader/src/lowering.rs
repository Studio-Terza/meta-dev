mod attributes;
mod boundary;
mod calls;
mod definition;
mod frames;
mod held;
mod interface;
mod laws;
mod places;
mod quotes;
mod regions;
mod seat;
mod signature;

mod uses;
mod walk;

use algebra::{Catenation, Monoid, Semigroup};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use syntax::Spanned;
use syntax::{Attribute, Ident, ItemImpl, ItemMod};

use graph::{
    Alias, Aliases, Assigned, Declarations, Entry, Evaluation, Exports, Exposure, FileId, FileTable, Function,
    Implementation, Interface, Model, ModelPart, Mutation, Named, Origin, Output, Owner, Region, Signature, Sited,
    Spelling, Subject, TypeRef,
};
use lint_core::{SourceFile, Syntax};
use lint_layout::{Name, Position};

use crate::namespace::{
    Declared, DeclaredKind, Domain, ItemIds, Namespace, Openings, Resolver, Resolving, ScopeId, ScopeKey, ScopeMap,
    ScopeOp, Seeds, Sowing, Sown, StdNames, Top, TranscribedRow, Transcriptions, UsePath, Visibility,
};
use crate::parallel::{Parallel, Parked};
use crate::planting::Unclaimed;
use attributes::Attributes;
use boundary::{SelfType, SlotOrdinal};
use calls::CallRow;
use definition::{BodyPart, LoweredDefinition};
use frames::Enclosure;
use held::HoldingRow;
use interface::TraitPath;
use laws::LawRow;
use lint_sweep::{Address, WalkPosition};
use places::{PlaceId, PlaceRows, Places};
use quotes::QuoteRow;
use regions::{Context, Regions};
use seat::{Footing, Located, Lowers, Seat};
pub use signature::ReceiverSyntax;
use signature::{DeclaredParameter, InScope, LoweredInput, LoweredParameter, Written};

use uses::UseRow;
pub(crate) use walk::Lowering;

#[derive(Debug)]
struct ImplRow {
    located: Located,
    subject: Name,
    ty: TypeRef<Written>,
    implemented: Option<UsePath>,
    arguments: Vec<TypeRef<Written>>,
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
        let within = scope.subject(ty.clone());
        ImplRow {
            located: seat.located(Position::from(block.impl_token.span)),
            subject: SelfType::written(&block.self_ty),
            ty,
            implemented: block.trait_.as_ref().map(|(_, path, _)| UsePath::from(path)),
            arguments: block.trait_.as_ref().map_or_default(|(_, path, _)| TraitPath::from(path).arguments(&within)),
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
            arguments: Vec::new(),
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

impl Lowers for ImplRow {
    fn lowered(self, footing: Footing<'_>) -> ModelPart {
        let located = self.located;
        let resolving = footing.resolving();
        let implemented = self.implemented.map(|path| {
            let spelling = Spelling::Written(path.written(), located.position());
            let scope = footing.scopes().scope(located.scope());
            let arguments = self.arguments.iter().map(|argument| Written::typed(argument, &resolving, scope)).collect();
            Interface::new(Sited::new(resolving.path(scope, &path, self.domain), spelling), arguments)
        });
        let subject =
            Subject::new(self.subject, Written::typed(&self.ty, &resolving, footing.scopes().scope(located.scope())));
        let implementation =
            Implementation::new(subject, implemented, self.origin, self.region.combine(footing.outer())).parameterised(
                self.parameters
                    .into_iter()
                    .map(|parameter| parameter.resolved(&resolving, footing.scopes().scope(located.scope())))
                    .collect(),
                self.assigned
                    .iter()
                    .map(|assigned| {
                        Assigned::new(
                            assigned.name.clone(),
                            Written::typed(&assigned.ty, &resolving, footing.scopes().scope(located.scope())),
                        )
                    })
                    .collect(),
            );
        located.implementation(footing, implementation)
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
    fn resolved(resolving: &Resolving<'_, '_, '_>, scope: ScopeId, output: Option<TypeRef<Written>>) -> Output {
        match output {
            Some(ty) => Output::Written(Written::typed(&ty, resolving, scope)),
            None => Output::Implicit,
        }
    }
}

impl Lowers for FnRow {
    fn lowered(self, footing: Footing<'_>) -> ModelPart {
        let scope = footing.scopes().scope(self.located.scope());
        let resolving = &footing.resolving();
        let inputs = self.inputs.into_iter().map(|input| input.typed(resolving, scope)).collect();
        let constraints = self.constraints.iter().map(|ty| Written::typed(ty, resolving, scope)).collect();
        let parameters = self.parameters.into_iter().map(|parameter| parameter.resolved(resolving, scope)).collect();
        let signature = Signature::new(inputs, Self::resolved(resolving, scope, self.output), constraints)
            .generic(parameters, self.mutation);
        let function = Function::new(self.name, self.owner, self.entry, signature)
            .exposed(self.exposure)
            .evaluated(self.evaluation)
            .seated(footing.places().seat(self.region.combine(footing.outer()), self.place.as_ref()))
            .defined(self.definition.map(|definition| definition.resolved(resolving, scope)));
        self.located.function(footing, function)
    }
}

#[derive(Debug)]
struct AliasRow {
    scope: ScopeKey,
    name: UsePath,
    parameters: Vec<DeclaredParameter>,
    target: TypeRef<Written>,
}

impl AliasRow {
    fn declared(self, resolving: &Resolving<'_, '_, '_>, scopes: &ScopeMap) -> Aliases {
        let scope = scopes.scope(&self.scope);
        let declared = resolving.path(scope, &self.name, Domain::Types).crate_item();
        let parameters = self.parameters.into_iter().map(|parameter| parameter.resolved(resolving, scope)).collect();
        let alias = Alias::new(parameters, Written::typed(&self.target, resolving, scope));
        declared.map_or_default(|item| Aliases::declaring(item, alias))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SlotForm {
    Conventional,
    Pathed,
}

impl From<&[Attribute]> for SlotForm {
    fn from(attributes: &[Attribute]) -> SlotForm {
        match attributes.iter().find(|attribute| attribute.path().is_ident(Slot::<ScopeId>::PATH)) {
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
pub(crate) struct Slot<Scope = ScopeId> {
    scope: Scope,
    name: Name,
    children: Arc<Path>,
    form: SlotForm,
    visibility: Visibility,
    region: Region,
}

impl<Scope> Slot<Scope> {
    const PATH: &'static str = "path";
    const EXTENSION: &'static str = "rs";
    const MODULE_FILE: &'static str = "mod.rs";

    pub(crate) fn found(&self, files: Unclaimed<'_, '_, '_>) -> Option<PathBuf> {
        let named = self.children.join(&self.name);
        let flat = named.with_extension(Self::EXTENSION);
        let nested = named.join(Self::MODULE_FILE);
        let found_flat = files.present(&flat).map(|_| flat);
        let found_nested = files.present(&nested).map(|_| nested);
        self.form.chosen(found_flat.xor(found_nested))
    }
}

impl Slot<ScopeKey> {
    fn new(seat: &Seat, region: Region, item: &ItemMod) -> Slot<ScopeKey> {
        Slot {
            scope: seat.scope().clone(),
            name: Name::from(&item.ident),
            children: seat.children(),
            form: SlotForm::from(item.attrs.as_slice()),
            visibility: Visibility::from(&item.vis),
            region: region.combine(Regions::of_module(item)),
        }
    }

    fn placed(self, base: ScopeId, scopes: &ScopeMap, outer: Region) -> Slot {
        Slot {
            scope: scopes.scope(&self.scope).above(base),
            name: self.name,
            children: self.children,
            form: self.form,
            visibility: self.visibility,
            region: self.region.combine(outer),
        }
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
    transcribed: Vec<TranscribedRow>,
}

impl Semigroup for FileRows {
    fn combine(self, other: FileRows) -> FileRows {
        FileRows {
            impls: self.impls.combine(other.impls),
            functions: self.functions.combine(other.functions),
            aliases: self.aliases.combine(other.aliases),
            holdings: self.holdings.combine(other.holdings),
            calls: self.calls.combine(other.calls),
            uses: self.uses.combine(other.uses),
            laws: self.laws.combine(other.laws),
            quotes: self.quotes.combine(other.quotes),
            places: self.places.combine(other.places),
            transcribed: self.transcribed.combine(other.transcribed),
        }
    }
}

impl Monoid for FileRows {}

impl Lowers for FileRows {
    fn lowered(self, footing: Footing<'_>) -> ModelPart {
        Monoid::concat([
            footing.every(self.impls),
            footing.every(self.functions),
            footing.every(self.holdings),
            footing.every(self.calls),
            footing.every(self.uses),
            footing.every(self.laws),
            footing.every(self.quotes),
        ])
    }
}

#[derive(Debug)]
struct Footed<'footing> {
    footing: Footing<'footing>,
    rows: FileRows,
}

#[derive(Debug)]
pub(crate) struct Lowered {
    openings: Openings,
    scopes: ScopeMap,
    slots: Vec<Slot<ScopeKey>>,
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
        let module = rooted.map_or(Some(path), |_| None).filter(|candidate| {
            candidate.file_name().is_none_or(|name| name != OsStr::new(Slot::<ScopeId>::MODULE_FILE))
        });
        let children = module.map_or_else(|| parent.to_path_buf(), |stem| stem.with_extension(""));
        FileShape { enclosure, children: Arc::from(children) }
    }
}

#[derive(Debug)]
struct Grown {
    file: FileId,
    base: ScopeId,
    outer: Region,
    rows: FileRows,
    scopes: ScopeMap,
}

impl Grown {
    fn settled(&self, resolving: &Resolving<'_, '_, '_>) -> Places {
        self.rows.places.settled(&resolving.based(self.base), &self.scopes)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct SlotAt {
    file: FileId,
    ordinal: SlotOrdinal,
}

impl SlotAt {
    pub(crate) fn every(file: FileId, lowered: &Lowered) -> impl Iterator<Item = SlotAt> + use<> {
        SlotOrdinal::every(&lowered.slots).map(move |ordinal| SlotAt { file, ordinal })
    }

    fn placed<'plot>(self, plot: &'plot Plot<'_>) -> Option<&'plot Slot> {
        plot.slots.get(&self)
    }
}

impl Lowered {
    pub(crate) fn slots(&self) -> &[Slot<ScopeKey>] {
        &self.slots
    }
}

#[derive(Debug, Default)]
pub(crate) struct Plot<'read> {
    space: Sown<'read>,
    grown: Vec<Grown>,
    slots: BTreeMap<SlotAt, Slot>,
}

impl Semigroup for Plot<'_> {
    fn combine(self, other: Self) -> Self {
        let mut slots = self.slots;
        slots.extend(other.slots);
        Plot { space: self.space.combine(other.space), grown: self.grown.combine(other.grown), slots }
    }
}

impl Monoid for Plot<'_> {}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Plotting<'plot, 'read> {
    plot: &'plot Plot<'read>,
}

impl<'plot, 'read> From<&'plot Plot<'read>> for Plotting<'plot, 'read> {
    fn from(plot: &'plot Plot<'read>) -> Plotting<'plot, 'read> {
        Plotting { plot }
    }
}

impl<'read> Plotting<'_, 'read> {
    pub(crate) fn rooted(self, top: Top<'read>, file: FileId, lowered: Lowered) -> Plot<'read> {
        self.grown_at(top, file, Region::SHIPPED, lowered)
    }

    pub(crate) fn planted(self, at: SlotAt, file: FileId, lowered: Lowered) -> Plot<'read> {
        at.placed(self.plot).map_or_default(|slot| {
            let top = Top::Child { parent: slot.scope, name: slot.name.clone(), visibility: slot.visibility };
            self.grown_at(top, file, slot.region, lowered)
        })
    }

    pub(crate) fn opaque(self, at: SlotAt) -> Plot<'read> {
        at.placed(self.plot).map_or_default(|slot| {
            let declared = Declared::new(slot.name.clone(), slot.visibility, DeclaredKind::OpaqueModule);
            Plot { space: Sown::declaring(slot.scope, declared), ..Plot::default() }
        })
    }

    fn grown_at(self, top: Top<'read>, file: FileId, outer: Region, lowered: Lowered) -> Plot<'read> {
        let sowing = Sowing::from(&self.plot.space);
        let base = sowing.base();
        let space = sowing.placed(top, lowered.openings);
        let ordinals = SlotOrdinal::every(&lowered.slots);
        let slots = ordinals
            .zip(lowered.slots)
            .map(|(ordinal, slot)| (SlotAt { file, ordinal }, slot.placed(base, &lowered.scopes, outer)))
            .collect();
        let grown = vec![Grown { file, base, outer, rows: lowered.rows, scopes: lowered.scopes }];
        Plot { space, grown, slots }
    }
}

impl Plot<'_> {
    pub(crate) fn built(self, files: FileTable) -> Model {
        let space = Namespace::from(self.space);
        let seeds = Seeds::from(&space);
        let settled = seeds.settled();
        let tables = seeds.tables(&settled);
        let issued = space.issued();
        let ids = ItemIds::from(&issued);
        let unexpanded = Transcriptions::default();
        let naming = Resolving::new(Resolver::new(&space, &tables), &ids, &unexpanded);
        let transcriptions = Transcriptions::settled(self.grown.iter().flat_map(|grown| {
            let based = naming.based(grown.base);
            grown.rows.transcribed.iter().filter_map(move |row| row.settled(based, &grown.scopes))
        }));
        let resolving = Resolving::new(Resolver::new(&space, &tables), &ids, &transcriptions);
        let exports: Exports = Monoid::concat(space.exports().iter().map(|export| export.recorded(&resolving)));
        let places: Vec<Places> = Parallel::available().mapped(&self.grown, |grown| grown.settled(&resolving));
        let declarations: ModelPart =
            Monoid::concat(self.grown.iter().zip(&places).map(|(grown, seated)| seated.placed(grown.file)));
        let mut planted = self.grown;
        let mut taken: Vec<FileRows> = planted.iter_mut().map(|file| mem::take(&mut file.rows)).collect();
        let declared: Aliases = Monoid::concat(taken.iter_mut().zip(&planted).flat_map(|(row, grown)| {
            let based = resolving.based(grown.base);
            mem::take(&mut row.aliases).into_iter().map(move |alias| alias.declared(&based, &grown.scopes))
        }));
        let aliases = StdNames::aliased().combine(declared);
        let parked: Vec<Parked<Footed<'_>>> = taken
            .into_iter()
            .zip(&planted)
            .zip(&places)
            .map(|((rows, grown), seated)| Footed {
                footing: Footing::new(resolving.based(grown.base), grown, seated, &aliases),
                rows,
            })
            .map(Parked::from)
            .collect();
        let fragments = Parallel::available().shares(&parked).worked(|share| {
            Monoid::concat(share.iter().filter_map(Parked::taken).map(|file| file.rows.lowered(file.footing)))
        });
        let part = declarations.combine(Monoid::concat(fragments));
        Model::lowered(files, Declarations::issued(issued, aliases, exports), part)
    }
}

#[derive(Clone, Debug)]
pub(super) struct Frame<'read> {
    source: &'read SourceFile,
    here: Seat,
    enclosure: Enclosure,
    generics: Arc<InScope>,
    context: Context,
    calling: Option<Address>,
    tabled: Option<()>,
    within: Arc<[WalkPosition]>,
    at: WalkPosition,
}

#[derive(Debug, Default)]
pub(crate) struct Framed<'read>(Option<Box<Frame<'read>>>);

impl Semigroup for Framed<'_> {
    fn combine(self, other: Self) -> Self {
        Framed(self.0.or(other.0))
    }
}

impl Monoid for Framed<'_> {}

impl<'read> Framed<'read> {
    pub(super) fn framing(frame: Frame<'read>) -> Framed<'read> {
        Framed(Some(Box::new(frame)))
    }
}
#[derive(Debug)]
pub(super) struct Owning {
    place: PlaceId,
    grains: Grains,
}

impl Owning {
    fn within(&self, context: &Context) -> Context {
        context.owned(self.place.clone())
    }
}

#[derive(Debug)]
enum Grain {
    Impl(ImplRow),
    Function(FnRow),
    Alias(AliasRow),
    Holding(HoldingRow),
    Call(CallRow),
    Use(UseRow),
    Law(LawRow),
    Quote(QuoteRow),
    Place(PlaceRows),
    Op(ScopeOp),
    Slot(Slot<ScopeKey>),
    Body(BodyPart),
    Transcribed(TranscribedRow),
}

#[derive(Debug, Default)]
pub(crate) struct Grains {
    grains: Catenation<Grain>,
    bodied: Option<()>,
}

impl Semigroup for Grains {
    fn combine(self, other: Grains) -> Grains {
        Grains { grains: self.grains.combine(other.grains), bodied: self.bodied.or(other.bodied) }
    }
}

impl Monoid for Grains {}

impl From<Grain> for Grains {
    fn from(grain: Grain) -> Grains {
        let bodied = grain.bodied();
        Grains { grains: Catenation::from(vec![grain]), bodied }
    }
}

#[derive(Debug, Default)]
struct Sorted {
    rows: FileRows,
    ops: Vec<ScopeOp>,
    slots: Vec<Slot<ScopeKey>>,
}

impl Semigroup for Sorted {
    fn combine(self, other: Sorted) -> Sorted {
        Sorted {
            rows: self.rows.combine(other.rows),
            ops: self.ops.combine(other.ops),
            slots: self.slots.combine(other.slots),
        }
    }
}

impl Monoid for Sorted {}

impl From<Grain> for Sorted {
    fn from(grain: Grain) -> Sorted {
        match grain {
            Grain::Impl(row) => Sorted::from(FileRows { impls: vec![row], ..FileRows::default() }),
            Grain::Function(row) => Sorted::from(FileRows { functions: vec![row], ..FileRows::default() }),
            Grain::Alias(row) => Sorted::from(FileRows { aliases: vec![row], ..FileRows::default() }),
            Grain::Holding(row) => Sorted::from(FileRows { holdings: vec![row], ..FileRows::default() }),
            Grain::Call(row) => Sorted::from(FileRows { calls: vec![row], ..FileRows::default() }),
            Grain::Use(row) => Sorted::from(FileRows { uses: vec![row], ..FileRows::default() }),
            Grain::Law(row) => Sorted::from(FileRows { laws: vec![row], ..FileRows::default() }),
            Grain::Quote(row) => Sorted::from(FileRows { quotes: vec![row], ..FileRows::default() }),
            Grain::Transcribed(row) => Sorted::from(FileRows { transcribed: vec![row], ..FileRows::default() }),
            Grain::Place(places) => Sorted::from(FileRows { places, ..FileRows::default() }),
            Grain::Op(op) => Sorted { ops: vec![op], ..Sorted::default() },
            Grain::Slot(slot) => Sorted { slots: vec![slot], ..Sorted::default() },
            Grain::Body(_) => Sorted::default(),
        }
    }
}

impl From<FileRows> for Sorted {
    fn from(rows: FileRows) -> Sorted {
        Sorted { rows, ..Sorted::default() }
    }
}

impl From<Grains> for Sorted {
    fn from(grains: Grains) -> Sorted {
        Monoid::concat(grains.grains.into_iter().map(Sorted::from))
    }
}

impl Grain {
    fn body(self) -> Option<BodyPart> {
        match self {
            Grain::Body(part) => Some(part),
            Grain::Impl(_)
            | Grain::Function(_)
            | Grain::Alias(_)
            | Grain::Holding(_)
            | Grain::Call(_)
            | Grain::Use(_)
            | Grain::Law(_)
            | Grain::Quote(_)
            | Grain::Transcribed(_)
            | Grain::Place(_)
            | Grain::Op(_)
            | Grain::Slot(_) => None,
        }
    }

    fn bodied(&self) -> Option<()> {
        match self {
            Grain::Body(_) => Some(()),
            Grain::Impl(_)
            | Grain::Function(_)
            | Grain::Alias(_)
            | Grain::Holding(_)
            | Grain::Call(_)
            | Grain::Use(_)
            | Grain::Law(_)
            | Grain::Quote(_)
            | Grain::Transcribed(_)
            | Grain::Place(_)
            | Grain::Op(_)
            | Grain::Slot(_) => None,
        }
    }

    fn function(&self) -> Option<&FnRow> {
        match self {
            Grain::Function(row) => Some(row),
            Grain::Impl(_)
            | Grain::Alias(_)
            | Grain::Holding(_)
            | Grain::Call(_)
            | Grain::Use(_)
            | Grain::Law(_)
            | Grain::Quote(_)
            | Grain::Transcribed(_)
            | Grain::Place(_)
            | Grain::Op(_)
            | Grain::Slot(_)
            | Grain::Body(_) => None,
        }
    }

    fn defined(self, definition: Option<LoweredDefinition>) -> Grain {
        match self {
            Grain::Function(row) => Grain::Function(FnRow { definition: definition.or(row.definition), ..row }),
            other => other,
        }
    }
}

impl From<BodyPart> for Grains {
    fn from(part: BodyPart) -> Grains {
        Grains::from(Grain::Body(part))
    }
}

impl Grains {
    fn closed(self) -> Grains {
        Grains { grains: self.grains.into_iter().filter(|grain| grain.bodied().is_none()).collect(), bodied: None }
    }
}
