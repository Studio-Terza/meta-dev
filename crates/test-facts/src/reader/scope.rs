use std::collections::BTreeMap;
use std::iter;
use std::mem;

use syn::visit::{self, Visit};
use syn::{
    Block, Expr, Ident, ImplItemConst, ImplItemFn, ItemConst, ItemFn, ItemImpl, ItemMod, ItemUse, Path, Signature,
    UseTree,
};

use lint_core::{MacroAliases, SourceFile, Syntax};
use lint_layout::{Name, Word};
use lint_placement::ModulePath;

use super::trace::Locals;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Route {
    names: Vec<Name>,
}

impl From<&ModulePath> for Route {
    fn from(module: &ModulePath) -> Route {
        Route { names: module.segments().to_vec() }
    }
}

impl Route {
    fn joined(&self, names: &[Name]) -> Route {
        Route { names: self.names.iter().chain(names).cloned().collect() }
    }

    fn spelled(&self) -> Name {
        Name::from(self.names.iter().map(Name::to_string).collect::<Vec<String>>().join("::").as_str())
    }

    fn parent(&self) -> Option<Route> {
        self.names.split_last().map(|(_, rest)| Route { names: rest.to_vec() })
    }
}

#[derive(Clone, Copy, Debug)]
enum Anchor {
    Crate,
    Current,
    Parent,
    Own,
}

impl Anchor {
    const SPELLINGS: [(Anchor, &'static str); 4] =
        [(Anchor::Crate, "crate"), (Anchor::Current, "self"), (Anchor::Parent, "super"), (Anchor::Own, "Self")];

    fn named(name: &Name) -> Option<Anchor> {
        Self::SPELLINGS.iter().find(|(_, spelling)| *name == *spelling).map(|(anchor, _)| *anchor)
    }

    fn route(self, place: &Place, tail: &[Name]) -> Option<Route> {
        match self {
            Anchor::Crate => Some(Route::default().joined(tail)),
            Anchor::Current => Some(place.module.joined(tail)),
            Anchor::Parent => place
                .module
                .parent()
                .map(Place::inside)
                .and_then(|above| above.anchored(tail).or(Some(above.module.joined(tail)))),
            Anchor::Own => Some(place.module.joined(&place.owner.iter().chain(tail).cloned().collect::<Vec<Name>>())),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Place {
    module: Route,
    owner: Option<Name>,
}

impl Place {
    const DECLARING: &'static str = "Self";

    pub(crate) fn in_module(module: &ModulePath) -> Place {
        Place { module: Route::from(module), owner: None }
    }

    fn inside(module: Route) -> Place {
        Place { module, owner: None }
    }

    fn owned_by(&self, owner: Option<Name>) -> Place {
        Place { module: self.module.clone(), owner }
    }

    fn declaring(&self, ident: &Ident) -> Option<Route> {
        self.anchored(&[Name::from(Self::DECLARING), Name::from(ident)])
    }

    pub(crate) fn anchored(&self, names: &[Name]) -> Option<Route> {
        names.split_first().and_then(|(head, tail)| Anchor::named(head).and_then(|anchor| anchor.route(self, tail)))
    }
}

#[derive(Debug)]
pub(crate) struct Helper<'file> {
    route: Route,
    place: Place,
    inputs: Vec<Option<Name>>,
    block: &'file Block,
}

impl<'file> Helper<'file> {
    fn declared(place: Place, signature: &'file Signature, block: &'file Block) -> Helper<'file> {
        let inputs = signature
            .inputs
            .iter()
            .filter_map(Syntax::typed)
            .map(|typed| Syntax::pat_ident(&typed.pat).map(|input| Name::from(&input.ident)))
            .collect();
        let route = place.declaring(&signature.ident).unwrap_or_default();
        Helper { route, place, inputs, block }
    }

    pub(crate) fn matching(&self, arguments: &[&Expr]) -> Option<&Helper<'file>> {
        Some(self).filter(|helper| helper.inputs.len() == arguments.len())
    }

    pub(crate) fn route(&self) -> &Route {
        &self.route
    }

    pub(crate) fn place(&self) -> &Place {
        &self.place
    }

    pub(crate) fn called(&self, arguments: &[&'file Expr], caller: &Locals<'file>) -> Locals<'file> {
        self.inputs.iter().zip(arguments).fold(Locals::default(), |locals, (input, argument)| {
            input.iter().fold(locals, |locals, name| locals.binding(name.clone(), caller.resolved(argument)))
        })
    }

    pub(crate) fn visited(&self, visitor: &mut impl Visit<'file>) {
        visitor.visit_block(self.block);
    }
}

#[derive(Debug, Default)]
struct Imports {
    named: BTreeMap<Route, BTreeMap<Name, Route>>,
    globs: BTreeMap<Route, Vec<Route>>,
}

impl Imports {
    pub(crate) fn reaching(&self, names: &[Name], module: &Route) -> Vec<Route> {
        let aliased = names.split_first().and_then(|(first, rest)| {
            self.named.get(module).and_then(|aliases| aliases.get(first)).map(|target| target.joined(rest))
        });
        aliased.into_iter().chain(self.globs.get(module).into_iter().flatten().map(|glob| glob.joined(names))).collect()
    }

    fn recorded(&mut self, tree: &UseTree, prefix: &[Name], place: &Place) {
        match tree {
            UseTree::Path(branch) => self.recorded(&branch.tree, &Self::extended(prefix, &branch.ident), place),
            UseTree::Name(leaf) => self.named(&leaf.ident, &Self::extended(prefix, &leaf.ident), place),
            UseTree::Rename(leaf) => self.named(&leaf.rename, &Self::extended(prefix, &leaf.ident), place),
            UseTree::Glob(_) => self.globbed(prefix, place),
            UseTree::Group(group) => group.items.iter().for_each(|item| self.recorded(item, prefix, place)),
        }
    }

    fn extended(prefix: &[Name], ident: &Ident) -> Vec<Name> {
        prefix.iter().cloned().chain(iter::once(Name::from(ident))).collect()
    }

    fn named(&mut self, alias: &Ident, path: &[Name], place: &Place) {
        let route = place.anchored(path).unwrap_or_else(|| place.module.joined(path));
        self.named.entry(place.module.clone()).or_default().insert(Name::from(alias), route);
    }

    fn globbed(&mut self, path: &[Name], place: &Place) {
        let route = place.anchored(path).unwrap_or_else(|| place.module.joined(path));
        self.globs.entry(place.module.clone()).or_default().push(route);
    }
}

#[derive(Debug)]
pub(crate) struct Scope<'file> {
    constants: BTreeMap<Route, &'file Expr>,
    helpers: BTreeMap<Route, Vec<Helper<'file>>>,
    imports: Imports,
    tables: MacroAliases,
    at: Place,
}

impl<'file> From<&'file SourceFile> for Scope<'file> {
    fn from(file: &'file SourceFile) -> Scope<'file> {
        let mut scope = Scope {
            constants: BTreeMap::new(),
            helpers: BTreeMap::new(),
            imports: Imports::default(),
            tables: MacroAliases::in_file(file.syntax(), Self::TABLE),
            at: Place::in_module(file.placement().module()),
        };
        scope.visit_file(file.syntax());
        scope
    }
}

impl<'file> Scope<'file> {
    const TABLE: Word<'static> = Word::new("table");

    pub(crate) fn tables(&self) -> &MacroAliases {
        &self.tables
    }

    pub(crate) fn constant(&self, path: &Path, place: &Place) -> Option<&'file Expr> {
        self.resolved(path, place).iter().find_map(|route| self.constants.get(route).copied())
    }

    pub(crate) fn helpers(&self, path: &Path, place: &Place, arguments: &[&Expr]) -> Vec<&Helper<'file>> {
        self.resolved(path, place)
            .iter()
            .find_map(|route| self.helpers.get(route))
            .into_iter()
            .flatten()
            .filter_map(|helper| helper.matching(arguments))
            .collect()
    }

    pub(crate) fn called(&self, path: &Path, place: &Place) -> Name {
        let written = Route { names: path.segments.iter().map(|segment| Name::from(&segment.ident)).collect() };
        let defined = self.resolved(path, place).into_iter().find(|route| self.helpers.contains_key(route));
        defined.unwrap_or(written).spelled()
    }

    fn resolved(&self, path: &Path, place: &Place) -> Vec<Route> {
        let names: Vec<Name> = path.segments.iter().map(|segment| Name::from(&segment.ident)).collect();
        let anchored = place.anchored(&names);
        let visible = iter::once(place.module.joined(&names)).chain(self.imports.reaching(&names, &place.module));
        anchored.map_or_else(|| visible.collect(), |route| vec![route])
    }

    fn register(&mut self, signature: &'file Signature, block: &'file Block) {
        let helper = Helper::declared(self.at.clone(), signature, block);
        self.helpers.entry(helper.route().clone()).or_default().push(helper);
    }
}

impl<'file> Visit<'file> for Scope<'file> {
    fn visit_item_mod(&mut self, item: &'file ItemMod) {
        let entered = Place::inside(self.at.module.joined(&[Name::from(&item.ident)]));
        let outer = mem::replace(&mut self.at, entered);
        visit::visit_item_mod(self, item);
        self.at = outer;
    }

    fn visit_item_use(&mut self, item: &'file ItemUse) {
        self.imports.recorded(&item.tree, &[], &self.at);
    }

    fn visit_item_const(&mut self, item: &'file ItemConst) {
        self.constants.extend(self.at.declaring(&item.ident).map(|route| (route, &*item.expr)));
    }

    fn visit_item_fn(&mut self, item: &'file ItemFn) {
        self.register(&item.sig, &item.block);
        visit::visit_item_fn(self, item);
    }

    fn visit_item_impl(&mut self, item: &'file ItemImpl) {
        let entered = self.at.owned_by(Name::of_type(&item.self_ty));
        let outer = mem::replace(&mut self.at, entered);
        visit::visit_item_impl(self, item);
        self.at = outer;
    }

    fn visit_impl_item_const(&mut self, item: &'file ImplItemConst) {
        self.constants.extend(self.at.declaring(&item.ident).map(|route| (route, &item.expr)));
    }

    fn visit_impl_item_fn(&mut self, item: &'file ImplItemFn) {
        Some(&item.sig).filter(|signature| signature.receiver().is_none()).iter().for_each(|signature| {
            self.register(signature, &item.block);
        });
        visit::visit_impl_item_fn(self, item);
    }
}
