use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use invariants::Ignoring;
use proc_macro2::{Ident, TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{
    Attribute, ImplItem, ItemConst, ItemEnum, ItemFn, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait,
    ItemType, ItemUnion, ItemUse, Visibility,
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
}

#[derive(Debug)]
pub(crate) struct Reading {
    reader: Reader,
    names: BTreeSet<Name>,
}

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
}

#[derive(Debug, Default)]
struct Read(BTreeMap<Reader, BTreeSet<Name>>);

impl DeadCode<'_> {
    const ENTRY_POINTS: [&'static str; 3] = ["proc_macro", "proc_macro_derive", "proc_macro_attribute"];
    const FIX: Fix = Fix("make it private or delete it");

    fn export(visibility: &Visibility, attributes: &[Attribute], name: &Ident) -> Option<Exported> {
        let entry = attributes
            .iter()
            .filter_map(|attribute| attribute.path().segments.last())
            .find(|segment| DeadCode::ENTRY_POINTS.iter().any(|entry| segment.ident == entry));
        let public = match visibility {
            Visibility::Public(_) => Some(()),
            Visibility::Restricted(_) | Visibility::Inherited => None,
        };
        let live = Gate::of(attributes).entered();
        let exported = public.and(live).and(entry.map_or(Some(()), |_| None));
        exported.map(|()| Exported { name: Name::of(name), line: Line::of(name.span()) })
    }

    fn reader(&self) -> Reader {
        let directory = Arc::from(self.source.package().directory());
        match (self.source.place(), self.source.own_crate()) {
            (Place::Shipped, Some(_)) => Reader::Library(directory),
            (Place::Shipped, None) => Reader::Binary(directory),
            (Place::Tests | Place::Elsewhere, _) => Reader::Tests(directory),
        }
    }

    fn names(tokens: TokenStream) -> BTreeSet<Name> {
        tokens
            .into_iter()
            .flat_map(|tree| match tree {
                TokenTree::Ident(ident) => BTreeSet::from([Name::of(&ident)]),
                TokenTree::Group(group) => DeadCode::names(group.stream()),
                TokenTree::Punct(_) | TokenTree::Literal(_) => BTreeSet::new(),
            })
            .collect()
    }
}

impl Read {
    fn named(&self, export: &Export) -> Option<()> {
        let crate_name = export.owner.crate_name();
        let readers =
            self.0.iter().filter(|(reader, names)| reader.reads(export).is_some() && names.contains(&crate_name));
        readers.into_iter().find(|(_, names)| names.contains(&export.exported.name)).map(|_| ())
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

impl<'ast> Visit<'ast> for DeadCode<'_> {
    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        self.exported.extend(DeadCode::export(&module.vis, &module.attrs, &module.ident));
        Gate::of(&module.attrs).entered().map_or_default(|()| visit::visit_item_mod(self, module));
    }

    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.sig.ident));
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident));
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident));
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident));
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident));
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident));
    }

    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident));
    }

    fn visit_item_static(&mut self, item: &'ast ItemStatic) {
        self.exported.extend(DeadCode::export(&item.vis, &item.attrs, &item.ident));
    }

    fn visit_item_macro(&mut self, _: &'ast ItemMacro) {}

    fn visit_item_use(&mut self, item: &'ast ItemUse) {
        let leaves = UseLeaf::of_tree(&item.tree);
        let bound = leaves.iter().filter_map(|leaf| leaf.end().bound());
        self.exported.extend(bound.filter_map(|name| DeadCode::export(&item.vis, &item.attrs, name)));
    }

    fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
        let inherent = Gate::inherent(item).and(Gate::of(&item.attrs)).entered();
        let items = inherent.map_or_default(|()| item.items.iter().collect::<Vec<&ImplItem>>());
        self.exported.extend(items.into_iter().filter_map(|inner| match inner {
            ImplItem::Fn(function) => DeadCode::export(&function.vis, &function.attrs, &function.sig.ident),
            ImplItem::Const(constant) => DeadCode::export(&constant.vis, &constant.attrs, &constant.ident),
            _ => None,
        }));
    }
}

impl<'file> Graph<'file> for DeadCode<'file> {
    const RULE: Rule = Rule::DeadCode;

    type Fact = DeadFact;

    fn new(source: &'file Source) -> DeadCode<'file> {
        DeadCode { source, exported: Vec::new() }
    }

    fn facts(self) -> Vec<DeadFact> {
        let shipped = (self.source.place() == Place::Shipped).then_some(());
        let tokens = TokenStream::from_str(self.source.text().as_ref()).ignoring(Ignored::Unparsed).unwrap_or_default();
        let reading = DeadFact::Reading(Reading { reader: self.reader(), names: DeadCode::names(tokens) });
        let owner = self.source.package();
        let path = self.source.shown();
        let home = self.reader();
        let exports = shipped.map_or_default(|()| self.exported).into_iter().map(|exported| {
            DeadFact::Export(Export { owner: Arc::clone(owner), home: home.clone(), exported, path: Arc::clone(path) })
        });
        [reading].into_iter().chain(exports).collect()
    }

    fn reduce(facts: Vec<DeadFact>, _: &[Arc<Package>]) -> Vec<Finding> {
        let (exports, readings): (Vec<Option<Export>>, Vec<Option<Reading>>) = facts
            .into_iter()
            .map(|fact| match fact {
                DeadFact::Export(export) => (Some(export), None),
                DeadFact::Reading(reading) => (None, Some(reading)),
            })
            .unzip();
        let read = readings.into_iter().flatten().fold(Read::default(), |mut read, reading| {
            read.0.entry(reading.reader).or_default().extend(reading.names);
            read
        });
        exports
            .into_iter()
            .flatten()
            .filter(|export| read.named(export).is_none())
            .map(|export| {
                let Exported { name, line } = &export.exported;
                Found::at(*line, format_args!("pub {name} is named by no other crate"), DeadCode::FIX)
                    .in_file(Rule::DeadCode, &export.path)
            })
            .collect()
    }
}
