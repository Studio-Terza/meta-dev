mod boundary;
mod surface;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use invariants::Ignoring;
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
    Attribute, Fields, ImplItem, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct,
    ItemTrait, ItemType, ItemUnion, ItemUse, Lit, Macro, Type, Visibility,
};

use crate::finding::{Finding, Found, Rule};
use crate::gate::Gate;
use crate::graph::Graph;
use crate::ignored::Ignored;
use crate::source::{Place, Source};
use crate::terms::{Fix, Line, Name};
use crate::use_leaf::UseLeaf;
use crate::walk::{Descent, Expanded, Hooks};
use crate::workspace::Package;
use boundary::{At, Indexed};
use surface::{Surface, SurfaceFact};

#[derive(Debug)]
pub(crate) enum DeadFact {
    Export(Export),
    Attachment(Attachment),
    Reading(Reading),
    Surface(SurfaceFact),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Spot {
    path: Arc<Path>,
    line: Line,
    name: Name,
}

#[derive(Debug)]
pub(crate) struct Export {
    owner: Arc<Package>,
    home: Reader,
    exported: Exported,
    path: Arc<Path>,
}

#[derive(Debug)]
pub(crate) struct Exported {
    name: Name,
    line: Line,
    found: Found,
    interface: Names,
}

#[derive(Debug)]
pub(crate) struct Attachment {
    owner: Arc<Package>,
    attached: Attached,
}

#[derive(Debug)]
pub(crate) struct Attached {
    to: Vec<Name>,
    interface: Names,
}

#[derive(Debug)]
pub(crate) struct Reading {
    reader: Reader,
    names: Names,
}

#[derive(Debug, Default)]
pub(crate) struct Names(BTreeSet<Name>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Reader {
    Library(Arc<Path>),
    Binary(Arc<Path>),
    Tests(Arc<Path>),
}

#[derive(Debug)]
pub(crate) struct DeadCode<'file> {
    source: &'file Source,
    exported: Vec<Exported>,
    attached: Vec<Attached>,
    held: Vec<Option<()>>,
    surface: Surface<'file>,
}

#[derive(Debug, Default)]
struct Read(BTreeMap<Reader, Names>);

#[derive(Debug, Default)]
struct Graphed {
    exports: Vec<Export>,
    attachments: Vec<Attachment>,
    read: Read,
    surface: Vec<SurfaceFact>,
}

impl Spot {
    fn new(path: Arc<Path>, line: Line, name: Name) -> Spot {
        Spot { path, line, name }
    }
}

impl DeadCode<'_> {
    const ENTRY_POINTS: [&'static str; 3] = ["proc_macro", "proc_macro_derive", "proc_macro_attribute"];
    const FIX: Fix = Fix("make it private or delete it");

    fn export(
        &self,
        visibility: &Visibility,
        attributes: &[Attribute],
        name: &Ident,
        interface: Names,
    ) -> Option<Exported> {
        let entry = attributes
            .iter()
            .filter_map(|attribute| attribute.path().segments.last())
            .find(|segment| DeadCode::ENTRY_POINTS.iter().any(|entry| segment.ident == entry));
        let live = Gate::of(attributes).entered();
        let exported =
            self.collecting().and(DeadCode::public(visibility)).and(live).and(entry.map_or(Some(()), |_| None));
        exported.map(|()| Exported {
            name: Name::of(name),
            line: Line::of(name.span()),
            found: Found::spanned(name.span(), format_args!("pub {name} is named by no other crate"), DeadCode::FIX),
            interface,
        })
    }

    fn exporting(&mut self, visibility: &Visibility, attributes: &[Attribute], name: &Ident, interface: Names) {
        let exported = self.export(visibility, attributes, name, interface);
        self.exported.extend(exported);
    }

    fn collecting(&self) -> Option<()> {
        self.held.iter().flatten().next().copied().xor(Some(()))
    }

    fn beneath(&mut self) -> Descent {
        self.held.push(Some(()));
        Descent::Into
    }

    fn surfaced(&mut self) {
        self.held.pop();
    }

    fn public(visibility: &Visibility) -> Option<()> {
        match visibility {
            Visibility::Public(_) => Some(()),
            Visibility::Restricted(_) | Visibility::Inherited => None,
        }
    }

    fn reader(&self) -> Reader {
        let directory = Arc::from(self.source.package().directory());
        match (self.source.place(), self.source.own_crate()) {
            (Place::Shipped, Some(_)) => Reader::Library(directory),
            (Place::Shipped, None) => Reader::Binary(directory),
            (Place::Tests | Place::Elsewhere, _) => Reader::Tests(directory),
        }
    }

    fn fielded(fields: &Fields) -> TokenStream {
        fields
            .iter()
            .filter(|field| DeadCode::public(&field.vis).is_some())
            .map(|field| field.ty.to_token_stream())
            .collect()
    }

    fn attached(item: &ItemImpl) -> Attached {
        let own = match &*item.self_ty {
            Type::Path(path) => path.path.segments.last().map(|segment| Name::of(&segment.ident)),
            _ => None,
        };
        let implemented = item.trait_.as_ref().and_then(|(_, path, _)| path.segments.last());
        let to = own.into_iter().chain(implemented.map(|segment| Name::of(&segment.ident))).collect();
        let trait_path = item.trait_.as_ref().map(|(_, path, _)| path.to_token_stream());
        let items = item.items.iter().map(|inner| match inner {
            ImplItem::Fn(function) => function.sig.to_token_stream(),
            ImplItem::Type(ty) => ty.ty.to_token_stream(),
            ImplItem::Const(constant) => constant.ty.to_token_stream(),
            _ => TokenStream::new(),
        });
        let written = [item.generics.to_token_stream()].into_iter().chain(trait_path).chain(items).collect();
        Attached { to, interface: Names::of(written) }
    }
}

impl Names {
    const FORMAT_IDENT: &'static str = "format_ident";
    const BANG: char = '!';

    fn of(tokens: TokenStream) -> Names {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let triples = trees.iter().zip(trees.iter().skip(1)).zip(trees.iter().skip(2));
        let formatted: BTreeSet<Name> = triples
            .filter_map(|triple| match triple {
                ((TokenTree::Ident(called), TokenTree::Punct(bang)), TokenTree::Group(group))
                    if *called == Names::FORMAT_IDENT && bang.as_char() == Names::BANG =>
                {
                    Names::spelled(group.stream())
                }
                _ => None,
            })
            .collect();
        let written = trees.into_iter().flat_map(|tree| match tree {
            TokenTree::Ident(ident) => BTreeSet::from([Name::of(&ident)]),
            TokenTree::Group(group) => Names::of(group.stream()).0,
            TokenTree::Punct(_) | TokenTree::Literal(_) => BTreeSet::new(),
        });
        Names(written.chain(formatted).collect())
    }

    fn spelled(arguments: TokenStream) -> Option<Name> {
        let first = arguments.into_iter().next()?;
        let literal = match first {
            TokenTree::Literal(literal) => Some(literal),
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Group(_) => None,
        }?;
        let spelled = match Lit::new(literal) {
            Lit::Str(text) => Some(text.value()),
            _ => None,
        }?;
        syn::parse_str::<Ident>(&spelled).ignoring(Ignored::Unparsed).map(|ident| Name::of(&ident))
    }
}

impl Read {
    fn named(&self, export: &Export) -> Option<()> {
        let crate_name = export.owner.crate_name();
        let readers =
            self.0.iter().filter(|(reader, names)| reader.reads(export).is_some() && names.0.contains(&crate_name));
        readers.into_iter().find(|(_, names)| names.0.contains(&export.exported.name)).map(|_| ())
    }
}

impl Reader {
    fn reads(&self, export: &Export) -> Option<()> {
        match self {
            Reader::Tests(directory) if **directory == *export.owner.directory() => None,
            Reader::Library(_) | Reader::Binary(_) | Reader::Tests(_) => (*self != export.home).then_some(()),
        }
    }
}

impl Graphed {
    fn absorb(&mut self, fact: DeadFact) {
        match fact {
            DeadFact::Export(export) => self.exports.push(export),
            DeadFact::Attachment(attachment) => self.attachments.push(attachment),
            DeadFact::Reading(reading) => self.read.0.entry(reading.reader).or_default().0.extend(reading.names.0),
            DeadFact::Surface(surfaced) => self.surface.push(surfaced),
        }
    }

    fn alive(&self) -> BTreeSet<At> {
        let named = self.indexed().filter(|indexed| self.read.named(indexed.export()).is_some());
        self.grown(named.map(Indexed::at).collect())
    }

    fn grown(&self, alive: BTreeSet<At>) -> BTreeSet<At> {
        let used: BTreeSet<At> =
            self.indexed().filter(|indexed| self.used(&alive, indexed.export()).is_some()).map(Indexed::at).collect();
        let reached: Vec<At> = used.difference(&alive).copied().collect();
        match reached.first() {
            Some(_) => self.grown(alive.into_iter().chain(reached).collect()),
            None => alive,
        }
    }

    fn used(&self, alive: &BTreeSet<At>, export: &Export) -> Option<()> {
        let directory = export.owner.directory();
        let living: Vec<&Export> =
            alive.iter().filter_map(|at| self.at(*at)).filter(|user| user.owner.directory() == directory).collect();
        let interfaced = living.iter().find(|user| user.exported.interface.0.contains(&export.exported.name));
        let attached =
            self.attachments.iter().filter(|attachment| attachment.owner.directory() == directory).find(|attachment| {
                attachment.attached.interface.0.contains(&export.exported.name)
                    && living.iter().any(|user| attachment.attached.to.contains(&user.exported.name))
            });
        interfaced.map(|_| ()).or(attached.map(|_| ()))
    }
}

impl Hooks for DeadCode<'_> {
    fn file(&mut self, file: &syn::File) -> Descent {
        Descent::from(self.surface.filed(file))
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Surface
    }

    fn item_use(&mut self, item: &ItemUse) -> Descent {
        let leaves = UseLeaf::of_tree(&item.tree);
        let exported: Vec<Exported> = leaves
            .iter()
            .filter_map(|leaf| leaf.end().bound())
            .filter_map(|name| self.export(&item.vis, &item.attrs, name, Names::default()))
            .collect();
        self.exported.extend(exported);
        self.surface.used(item);
        Descent::Over
    }

    fn path(&mut self, path: &syn::Path) -> Descent {
        self.surface.pathed(path);
        Descent::Into
    }

    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        self.exporting(&module.vis, &module.attrs, &module.ident, Names::default());
        self.held.push(Gate::of(&module.attrs).entered().xor(Some(())));
        Descent::Into
    }

    fn left_item_mod(&mut self, _: &ItemMod) {
        self.surfaced();
    }

    fn item_macro(&mut self, _: &ItemMacro) -> Descent {
        self.beneath()
    }

    fn left_item_macro(&mut self, _: &ItemMacro) {
        self.surfaced();
    }

    fn item_fn(&mut self, item: &ItemFn) -> Descent {
        let interface = Names::of(item.sig.to_token_stream());
        self.exporting(&item.vis, &item.attrs, &item.sig.ident, interface);
        self.beneath()
    }

    fn left_item_fn(&mut self, _: &ItemFn) {
        self.surfaced();
    }

    fn item_struct(&mut self, item: &ItemStruct) -> Descent {
        let tokens = [item.generics.to_token_stream(), DeadCode::fielded(&item.fields)].into_iter().collect();
        self.exporting(&item.vis, &item.attrs, &item.ident, Names::of(tokens));
        self.beneath()
    }

    fn left_item_struct(&mut self, _: &ItemStruct) {
        self.surfaced();
    }

    fn item_enum(&mut self, item: &ItemEnum) -> Descent {
        let interface = Names::of(item.to_token_stream());
        self.exporting(&item.vis, &item.attrs, &item.ident, interface);
        self.beneath()
    }

    fn left_item_enum(&mut self, _: &ItemEnum) {
        self.surfaced();
    }

    fn item_union(&mut self, item: &ItemUnion) -> Descent {
        let tokens = item.fields.named.iter().map(|field| field.ty.to_token_stream()).collect();
        self.exporting(&item.vis, &item.attrs, &item.ident, Names::of(tokens));
        self.beneath()
    }

    fn left_item_union(&mut self, _: &ItemUnion) {
        self.surfaced();
    }

    fn item_trait(&mut self, item: &ItemTrait) -> Descent {
        let interface = Names::of(item.to_token_stream());
        self.exporting(&item.vis, &item.attrs, &item.ident, interface);
        self.beneath()
    }

    fn left_item_trait(&mut self, _: &ItemTrait) {
        self.surfaced();
    }

    fn item_type(&mut self, item: &ItemType) -> Descent {
        let interface = Names::of(item.ty.to_token_stream());
        self.exporting(&item.vis, &item.attrs, &item.ident, interface);
        self.beneath()
    }

    fn left_item_type(&mut self, _: &ItemType) {
        self.surfaced();
    }

    fn item_const(&mut self, item: &ItemConst) -> Descent {
        let interface = Names::of(item.ty.to_token_stream());
        self.exporting(&item.vis, &item.attrs, &item.ident, interface);
        self.beneath()
    }

    fn left_item_const(&mut self, _: &ItemConst) {
        self.surfaced();
    }

    fn item_static(&mut self, item: &ItemStatic) -> Descent {
        let interface = Names::of(item.ty.to_token_stream());
        self.exporting(&item.vis, &item.attrs, &item.ident, interface);
        self.beneath()
    }

    fn left_item_static(&mut self, _: &ItemStatic) {
        self.surfaced();
    }

    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        let gated = Gate::of(&item.attrs).entered();
        let inherent = Gate::inherent(item).and(Gate::of(&item.attrs)).entered();
        let items = inherent.map_or_default(|()| item.items.iter().collect::<Vec<&ImplItem>>());
        let exported: Vec<Exported> = items
            .into_iter()
            .filter_map(|inner| match inner {
                ImplItem::Fn(function) => {
                    let interface = Names::of(function.sig.to_token_stream());
                    self.export(&function.vis, &function.attrs, &function.sig.ident, interface)
                }
                ImplItem::Const(constant) => {
                    let interface = Names::of(constant.ty.to_token_stream());
                    self.export(&constant.vis, &constant.attrs, &constant.ident, interface)
                }
                _ => None,
            })
            .collect();
        self.exported.extend(exported);
        let traited = self.collecting().and(Gate::inherent(item).entered().xor(Some(()))).and(gated);
        self.attached.extend(traited.map(|()| DeadCode::attached(item)));
        self.beneath()
    }

    fn left_item_impl(&mut self, _: &ItemImpl) {
        self.surfaced();
    }
}

impl<'file> Graph<'file> for DeadCode<'file> {
    type Fact = DeadFact;

    fn new(source: &'file Source) -> DeadCode<'file> {
        DeadCode { source, exported: Vec::new(), attached: Vec::new(), held: Vec::new(), surface: Surface::new(source) }
    }

    fn facts(self) -> Vec<DeadFact> {
        let shipped = (self.source.place() == Place::Shipped).then_some(());
        let tokens = TokenStream::from_str(self.source.text().as_str()).ignoring(Ignored::Unparsed).unwrap_or_default();
        let reading = DeadFact::Reading(Reading { reader: self.reader(), names: Names::of(tokens) });
        let owner = self.source.package();
        let path = self.source.shown();
        let home = self.reader();
        let DeadCode { exported: all_exported, attached: all_attached, surface, .. } = self;
        let exports = shipped.map_or_default(|()| all_exported).into_iter().map(|exported| {
            DeadFact::Export(Export { owner: Arc::clone(owner), home: home.clone(), exported, path: Arc::clone(path) })
        });
        let attachments = shipped
            .map_or_default(|()| all_attached)
            .into_iter()
            .map(|attached| DeadFact::Attachment(Attachment { owner: Arc::clone(owner), attached }));
        let surfaced = surface.facts().into_iter().map(DeadFact::Surface);
        [reading].into_iter().chain(exports).chain(attachments).chain(surfaced).collect()
    }

    fn reduce(facts: Vec<DeadFact>, packages: &[Arc<Package>]) -> Vec<Finding> {
        let graphed = facts.into_iter().fold(Graphed::default(), |mut graphed, fact| {
            graphed.absorb(fact);
            graphed
        });
        let alive = graphed.alive();
        let dead: Vec<&Export> = graphed.dead(&alive).collect();
        let spots: BTreeSet<Spot> = dead
            .iter()
            .map(|export| Spot::new(Arc::clone(&export.path), export.exported.line, export.exported.name.clone()))
            .collect();
        let unnamed = Surface::reduce(&graphed.surface, packages, &spots);
        dead.into_iter()
            .map(|export| export.exported.found.clone().in_file(Rule::DeadCode, &export.path))
            .chain(unnamed)
            .collect()
    }
}
