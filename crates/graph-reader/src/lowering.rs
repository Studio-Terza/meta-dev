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
mod walk;

use algebra::{Monoid, Semigroup};
use std::ffi::OsStr;
use std::mem;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use syntax::Spanned;
use syntax::{Attribute, Ident, ItemImpl, ItemMod};

use graph::{
    Alias, Assigned, Building, Entry, Evaluation, Exposure, FileId, Function, Implementation, Model, Mutation, Named,
    Origin, Output, Owner, Region, Signature, Sited, Spelling, Subject, TypeRef,
};
use lint_core::{SourceFile, Syntax};
use lint_layout::{Name, Position};

use crate::namespace::{
    Declared, DeclaredKind, Domain, Namespace, Openings, Resolver, Resolving, ScopeId, ScopeKey, ScopeMap, ScopeOp,
    Seeds, StdNames, Top, UsePath, Visibility,
};
use crate::parallel::{Parallel, Parked};
use crate::planting::Files;
use attributes::Attributes;
use boundary::SelfType;
use calls::CallRow;
use definition::LoweredDefinition;
use frames::Enclosure;
use held::HoldingRow;
use laws::LawRow;
use lint_sweep::Address;
use places::{PlaceId, PlaceRows, Places};
use quotes::QuoteRow;
use regions::{Context, Regions};
use seat::{Built, Footing, Located, Seat};
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
            Sited::new(resolving.path(footing.scope(located.scope()), &path, self.domain), spelling)
        });
        let subject = Subject::new(self.subject, Written::typed(&self.ty, &resolving, footing.scope(located.scope())));
        let implementation =
            Implementation::new(subject, implemented, self.origin, self.region.joined(footing.outer())).parameterised(
                self.parameters
                    .into_iter()
                    .map(|parameter| parameter.resolved(&resolving, footing.scope(located.scope())))
                    .collect(),
                self.assigned
                    .iter()
                    .map(|assigned| {
                        Assigned::new(
                            assigned.name.clone(),
                            Written::typed(&assigned.ty, &resolving, footing.scope(located.scope())),
                        )
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
    fn resolved(resolving: &Resolving<'_, '_, '_>, scope: ScopeId, output: Option<TypeRef<Written>>) -> Output {
        match output {
            Some(ty) => Output::Written(Written::typed(&ty, resolving, scope)),
            None => Output::Implicit,
        }
    }
}

impl Built for FnRow {
    fn built(self, footing: Footing<'_>, building: Building) -> Building {
        let scope = footing.scope(self.located.scope());
        let resolving = &footing.resolving();
        let inputs = self.inputs.into_iter().map(|input| input.typed(resolving, scope)).collect();
        let constraints = self.constraints.iter().map(|ty| Written::typed(ty, resolving, scope)).collect();
        let parameters = self.parameters.into_iter().map(|parameter| parameter.resolved(resolving, scope)).collect();
        let signature = Signature::new(inputs, Self::resolved(resolving, scope, self.output), constraints)
            .generic(parameters, self.mutation);
        let function = Function::new(self.name, self.owner, self.entry, signature)
            .exposed(self.exposure)
            .evaluated(self.evaluation)
            .seated(footing.places().seat(self.region.joined(footing.outer()), self.place.as_ref()))
            .defined(self.definition.map(|definition| definition.resolved(resolving, scope)));
        self.located.function(footing, building, function)
    }
}

#[derive(Debug)]
struct AliasRow {
    scope: ScopeKey,
    name: UsePath,
    parameters: Vec<DeclaredParameter>,
    target: TypeRef<Written>,
}

impl Built for AliasRow {
    fn built(self, footing: Footing<'_>, mut building: Building) -> Building {
        let resolving = &footing.resolving();
        let scope = footing.scope(&self.scope);
        let declared = resolving.path(scope, &self.name, Domain::Types).crate_item();
        let parameters = self.parameters.into_iter().map(|parameter| parameter.resolved(resolving, scope)).collect();
        let alias = Alias::new(parameters, Written::typed(&self.target, resolving, scope));
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

    pub(crate) fn found(&self, files: &Files<'_>) -> Option<PathBuf> {
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
            region: Regions::of_module(region, item),
        }
    }

    fn placed(self, base: ScopeId, scopes: &ScopeMap, outer: Region) -> Slot {
        Slot {
            scope: scopes.scope(&self.scope).above(base),
            name: self.name,
            children: self.children,
            form: self.form,
            visibility: self.visibility,
            region: self.region.joined(outer),
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
        }
    }
}

impl Monoid for FileRows {}

impl FileRows {
    fn defined(self, definition: Option<LoweredDefinition>) -> FileRows {
        let mut functions = self.functions.into_iter();
        let own = functions.next().map(|function| FnRow { definition: definition.or(function.definition), ..function });
        FileRows { functions: own.into_iter().chain(functions).collect(), ..self }
    }

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
        let slots = lowered.slots.into_iter().map(|slot| slot.placed(base, &lowered.scopes, outer)).collect();
        self.grown.push(Grown { file, base, outer, rows: lowered.rows, scopes: lowered.scopes });
        slots
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
        let places: Vec<Places> = Parallel::available()
            .mapped(&self.grown, |grown| grown.rows.places.settled(&resolving.based(grown.base), &grown.scopes));
        let with_places = self
            .grown
            .iter()
            .zip(&places)
            .fold(with_exports, |built, (grown, seated)| seated.placed(grown.file, built));
        let mut grown = self.grown;
        let mut footed: Vec<Footed<'_, FileRows>> = grown
            .iter_mut()
            .zip(&places)
            .map(|(file, seated)| Footed {
                footing: Footing::new(resolving.based(file.base), file.file, file.outer, seated, &file.scopes),
                row: mem::take(&mut file.rows),
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

#[derive(Clone, Debug)]
pub(super) struct Frame<'read> {
    source: &'read SourceFile,
    here: Seat,
    enclosure: Enclosure,
    generics: Arc<InScope>,
    context: Context,
    calling: Option<Address>,
    tabled: Option<()>,
    within: Arc<[Address]>,
}

#[derive(Debug, Default)]
pub(crate) struct Framed<'read>(Option<Frame<'read>>);

impl Semigroup for Framed<'_> {
    fn combine(self, other: Self) -> Self {
        Framed(self.0.or(other.0))
    }
}

impl Monoid for Framed<'_> {}

#[derive(Debug, Default)]
pub(crate) struct Grains {
    rows: FileRows,
    ops: Vec<ScopeOp>,
    slots: Vec<Slot<ScopeKey>>,
}

impl Semigroup for Grains {
    fn combine(self, other: Grains) -> Grains {
        Grains {
            rows: self.rows.combine(other.rows),
            ops: self.ops.combine(other.ops),
            slots: self.slots.combine(other.slots),
        }
    }
}

impl Monoid for Grains {}

impl From<FileRows> for Grains {
    fn from(rows: FileRows) -> Grains {
        Grains { rows, ..Grains::default() }
    }
}

impl<'read> Framed<'read> {
    pub(super) fn framing(frame: Frame<'read>) -> Framed<'read> {
        Framed(Some(frame))
    }
}
#[derive(Debug)]
pub(super) struct Owning {
    place: PlaceId,
    grains: Grains,
}
