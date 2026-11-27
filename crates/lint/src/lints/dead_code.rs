use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use invariants::Ignoring;
use proc_macro2::{Ident, TokenStream, TokenTree};
use quote::ToTokens;
use syn::visit::{self, Visit};
use syn::{
    Attribute, Fields, ImplItem, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct,
    ItemTrait, ItemType, ItemUnion, ItemUse, Lit, Type, Visibility,
};

use crate::finding::{Finding, Fix, Found, Line, Rule};
use crate::gate::Gate;
use crate::graph::Graph;
use crate::ignored::Ignored;
use crate::name::Name;
use crate::source::{Place, Source};
use crate::use_leaf::UseLeaf;
use crate::workspace::Package;

#[derive(Debug)]
pub(crate) enum DeadFact {
    Export(Export),
    Attachment(Attachment),
    Reading(Reading),
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
}

#[derive(Debug, Default)]
struct Read(BTreeMap<Reader, Names>);

#[derive(Debug, Default)]
struct Graphed {
    exports: Vec<Export>,
    attachments: Vec<Attachment>,
    read: Read,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct At(usize);

impl DeadCode<'_> {
    const ENTRY_POINTS: [&'static str; 3] = ["proc_macro", "proc_macro_derive", "proc_macro_attribute"];
    const FIX: Fix = Fix("make it private or delete it");

    fn export(visibility: &Visibility, attributes: &[Attribute], name: &Ident, interface: Names) -> Option<Exported> {
        let entry = attributes
            .iter()
            .filter_map(|attribute| attribute.path().segments.last())
            .find(|segment| DeadCode::ENTRY_POINTS.iter().any(|entry| segment.ident == entry));
        let live = Gate::of(attributes).entered();
        let exported = DeadCode::public(visibility).and(live).and(entry.map_or(Some(()), |_| None));
        exported.map(|()| Exported { name: Name::of(name), line: Line::of(name.span()), interface })
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
        }
    }

    fn alive(&self) -> BTreeSet<At> {
        let named = self.exports.iter().enumerate().filter(|(_, export)| self.read.named(export).is_some());
        self.grown(named.map(|(at, _)| At(at)).collect())
    }

    fn grown(&self, alive: BTreeSet<At>) -> BTreeSet<At> {
        let reached: BTreeSet<At> = self
            .exports
            .iter()
            .enumerate()
            .map(|(at, export)| (At(at), export))
            .filter(|(at, export)| !alive.contains(at) && self.used(&alive, export).is_some())
            .map(|(at, _)| at)
            .collect();
        match reached.first() {
            Some(_) => self.grown(alive.into_iter().chain(reached).collect()),
            None => alive,
        }
    }

    fn used(&self, alive: &BTreeSet<At>, export: &Export) -> Option<()> {
        let directory = export.owner.directory();
        let living: Vec<&Export> = alive
            .iter()
            .filter_map(|at| self.exports.get(at.0))
            .filter(|user| user.owner.directory() == directory)
            .collect();
        let interfaced = living.iter().find(|user| user.exported.interface.0.contains(&export.exported.name));
        let attached =
            self.attachments.iter().filter(|attachment| attachment.owner.directory() == directory).find(|attachment| {
                attachment.attached.interface.0.contains(&export.exported.name)
                    && living.iter().any(|user| attachment.attached.to.contains(&user.exported.name))
            });
        interfaced.map(|_| ()).or(attached.map(|_| ()))
    }
}

impl<'ast> Visit<'ast> for DeadCode<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.exported.extend(DeadCode::export(&module.vis, &module.attrs, &module.ident, Names::default()));
        Gate::of(&module.attrs).entered().map_or_default(|()| visit::visit_item_mod(self, module));
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        let interface = Names::of(item.sig.to_token_stream());
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.sig.ident, interface));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        let tokens = [item.generics.to_token_stream(), DeadCode::fielded(&item.fields)].into_iter().collect();
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident, Names::of(tokens)));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        let interface = Names::of(item.to_token_stream());
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident, interface));
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        let tokens = item.fields.named.iter().map(|field| field.ty.to_token_stream()).collect();
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident, Names::of(tokens)));
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        let interface = Names::of(item.to_token_stream());
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident, interface));
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        let interface = Names::of(item.ty.to_token_stream());
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident, interface));
    }

    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        let interface = Names::of(item.ty.to_token_stream());
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident, interface));
    }

    fn visit_item_static(&mut self, item: &'ast ItemStatic) {
        let interface = Names::of(item.ty.to_token_stream());
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident, interface));
    }

    fn visit_item_macro(&mut self, _: &'ast ItemMacro) {}

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        let leaves = UseLeaf::of_tree(&item.tree);
        let bound = leaves.iter().filter_map(|leaf| leaf.end().bound());
        self.exported.extend(bound.filter_map(|name| DeadCode::export(&item.vis, &item.attrs, name, Names::default())));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let gated = Gate::of(&item.attrs).entered();
        let inherent = Gate::inherent(item).and(Gate::of(&item.attrs)).entered();
        let items = inherent.map_or_default(|()| item.items.iter().collect::<Vec<&ImplItem>>());
        self.exported.extend(items.into_iter().filter_map(|inner| match inner {
            ImplItem::Fn(function) => {
                let interface = Names::of(function.sig.to_token_stream());
                DeadCode::export(&function.vis, &function.attrs, &function.sig.ident, interface)
            }
            ImplItem::Const(constant) => {
                let interface = Names::of(constant.ty.to_token_stream());
                DeadCode::export(&constant.vis, &constant.attrs, &constant.ident, interface)
            }
            _ => None,
        }));
        let traited = Gate::inherent(item).entered().xor(Some(())).and(gated);
        self.attached.extend(traited.map(|()| DeadCode::attached(item)));
    }
}

impl<'file> Graph<'file> for DeadCode<'file> {
    const RULE: Rule = Rule::DeadCode;

    type Fact = DeadFact;

    fn new(source: &'file Source) -> DeadCode<'file> {
        DeadCode { source, exported: Vec::new(), attached: Vec::new() }
    }

    fn facts(self) -> Vec<DeadFact> {
        let shipped = (self.source.place() == Place::Shipped).then_some(());
        let tokens = TokenStream::from_str(self.source.text().as_ref()).ignoring(Ignored::Unparsed).unwrap_or_default();
        let reading = DeadFact::Reading(Reading { reader: self.reader(), names: Names::of(tokens) });
        let owner = self.source.package();
        let path = self.source.shown();
        let home = self.reader();
        let DeadCode { exported: all_exported, attached: all_attached, .. } = self;
        let exports = shipped.map_or_default(|()| all_exported).into_iter().map(|exported| {
            DeadFact::Export(Export { owner: Arc::clone(owner), home: home.clone(), exported, path: Arc::clone(path) })
        });
        let attachments = shipped
            .map_or_default(|()| all_attached)
            .into_iter()
            .map(|attached| DeadFact::Attachment(Attachment { owner: Arc::clone(owner), attached }));
        [reading].into_iter().chain(exports).chain(attachments).collect()
    }

    fn reduce(facts: Vec<DeadFact>, _: &[Arc<Package>]) -> Vec<Finding> {
        let graphed = facts.into_iter().fold(Graphed::default(), |mut graphed, fact| {
            graphed.absorb(fact);
            graphed
        });
        let alive = graphed.alive();
        graphed
            .exports
            .iter()
            .enumerate()
            .filter(|(at, _)| !alive.contains(&At(*at)))
            .map(|(_, export)| {
                let Exported { name, line, .. } = &export.exported;
                Found::at(*line, format_args!("pub {name} is named by no other crate"), DeadCode::FIX)
                    .in_file(Rule::DeadCode, &export.path)
            })
            .collect()
    }
}
