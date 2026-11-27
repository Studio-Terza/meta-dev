use std::collections::BTreeSet;

use proc_macro2::{Delimiter, Ident, Punct, Spacing, TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{
    Attribute, Block, ImplItemFn, ItemConst, ItemEnum, ItemFn, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType,
    ItemUnion, Lit, Macro, TraitItemFn, TypePath,
};

use crate::finding::{Fix, Found, Rule};
use crate::gate::{Gate, TestMark};
use crate::source::{Place, Source};
use crate::style::{Reads, Style};

#[derive(Debug)]
pub(crate) struct TestShape {
    found: Vec<Found>,
    tested: Option<()>,
    rows: BTreeSet<Ident>,
}

#[derive(Debug, Default)]
struct Literals(Option<()>);

#[derive(Debug, Default)]
struct Named(BTreeSet<Ident>);

#[derive(Debug)]
struct Shape {
    name: Ident,
    named: Named,
}

#[derive(Debug, Default)]
struct Shapes {
    tables: Named,
    types: Vec<Shape>,
}

#[derive(Debug)]
enum Scan {
    Idle,
    Marked,
    Declared,
    Named(Ident),
}

#[derive(Clone, Copy, Debug)]
enum Helper {
    Function,
    Type,
}

impl TestShape {
    const LITERAL_FIX: Fix =
        Fix("read expected values from a golden file and inputs from a fixture or a proptest strategy");
    const HELPER_FIX: Fix = Fix("test through the production code path; a test file holds tests and their table");
    const DECLARING: &'static str = "fn";
    const MARK: char = '#';

    fn helper(&mut self, kind: Helper, name: &Ident) {
        let problem = match kind {
            Helper::Function => format!("test helper fn {name}"),
            Helper::Type => format!("test helper type {name}"),
        };
        let found = self.tested.map(|()| Found::spanned(name.span(), problem, TestShape::HELPER_FIX));
        self.found.extend(found);
    }

    fn row(&mut self, name: &Ident) {
        let row = self.rows.contains(name).then_some(());
        row.xor(Some(())).map_or_default(|()| self.helper(Helper::Type, name));
    }

    fn function(&mut self, attributes: &[Attribute], name: &Ident, block: &Block) {
        match TestMark::among(attributes) {
            Some(_) => {
                let mut literals = Literals::default();
                literals.visit_block(block);
                self.literal(name, literals.0);
            }
            None => self.helper(Helper::Function, name),
        }
    }

    fn literal(&mut self, name: &Ident, literal: Option<()>) {
        let found = literal
            .map(|()| Found::spanned(name.span(), format_args!("literal in test {name}"), TestShape::LITERAL_FIX));
        self.found.extend(found);
    }

    fn written(&mut self, tokens: TokenStream) -> Scan {
        tokens.into_iter().fold(Scan::Idle, |scanned, tree| self.scanned(scanned, tree))
    }

    fn scanned(&mut self, scan: Scan, tree: TokenTree) -> Scan {
        match (scan, tree) {
            (Scan::Named(name), TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                self.literal(&name, Literals::tokens(group.stream()));
                Scan::Idle
            }
            (Scan::Marked, TokenTree::Ident(ident)) if ident == TestShape::DECLARING => Scan::Declared,
            (Scan::Declared, TokenTree::Ident(ident)) => Scan::Named(ident),
            (Scan::Idle, TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                self.written(group.stream());
                Scan::Idle
            }
            (waiting, TokenTree::Group(group)) if group.delimiter() == Delimiter::Bracket => {
                let mark = TokenStream::from_iter([
                    TokenTree::Punct(Punct::new(TestShape::MARK, Spacing::Alone)),
                    TokenTree::Group(group),
                ]);
                TestMark::written(mark).iter().find_map(TestMark::on).map_or(waiting, |_| Scan::Marked)
            }
            (waiting, _) => waiting,
        }
    }
}

impl Literals {
    const TRUTHS: [&'static str; 2] = ["true", "false"];

    fn tokens(stream: TokenStream) -> Option<()> {
        stream.into_iter().find_map(|tree| match tree {
            TokenTree::Literal(_) => Some(()),
            TokenTree::Ident(ident) => Literals::TRUTHS.iter().find(|truth| ident == *truth).map(|_| ()),
            TokenTree::Group(group) => Literals::tokens(group.stream()),
            TokenTree::Punct(_) => None,
        })
    }
}

impl<'ast> Visit<'ast> for Literals {
    fn visit_lit(&mut self, _: &'ast Lit) {
        self.0 = Some(());
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.0 = self.0.or(Literals::tokens(invocation.tokens.clone()));
    }
}

impl<'ast> Visit<'ast> for Named {
    fn visit_type_path(&mut self, path: &'ast TypePath) {
        self.0.extend(path.path.segments.iter().map(|segment| segment.ident.clone()));
        visit::visit_type_path(self, path);
    }
}

impl<'ast> Visit<'ast> for Shapes {
    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.tables.visit_type(&item.ty);
        visit::visit_item_const(self, item);
    }

    fn visit_item_static(&mut self, item: &'ast ItemStatic) {
        self.tables.visit_type(&item.ty);
        visit::visit_item_static(self, item);
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        let mut named = Named::default();
        named.visit_fields(&item.fields);
        self.types.push(Shape { name: item.ident.clone(), named });
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        let mut named = Named::default();
        item.variants.iter().for_each(|variant| named.visit_fields(&variant.fields));
        self.types.push(Shape { name: item.ident.clone(), named });
    }
}

impl Shapes {
    fn rows(self) -> BTreeSet<Ident> {
        let Shapes { tables, types } = self;
        Shapes::grown(tables.0, &types)
    }

    fn grown(rows: BTreeSet<Ident>, types: &[Shape]) -> BTreeSet<Ident> {
        let reached: BTreeSet<Ident> = types
            .iter()
            .filter(|shape| rows.contains(&shape.name))
            .flat_map(|shape| shape.named.0.iter().filter(|inner| !rows.contains(*inner)).cloned())
            .collect();
        match reached.first() {
            Some(_) => Shapes::grown(rows.into_iter().chain(reached).collect(), types),
            None => rows,
        }
    }
}

impl<'ast> Visit<'ast> for TestShape {
    fn visit_file(&mut self, file: &'ast syn::File) {
        let mut shapes = Shapes::default();
        shapes.visit_file(file);
        self.rows = shapes.rows();
        visit::visit_file(self, file);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        let outer = self.tested;
        self.tested = outer.or(Gate::of(&module.attrs).entered().xor(Some(())));
        visit::visit_item_mod(self, module);
        self.tested = outer;
    }

    fn visit_item_fn(&mut self, function: &'ast ItemFn) {
        self.function(&function.attrs, &function.sig.ident, &function.block);
        visit::visit_item_fn(self, function);
    }

    fn visit_impl_item_fn(&mut self, function: &'ast ImplItemFn) {
        self.function(&function.attrs, &function.sig.ident, &function.block);
        visit::visit_impl_item_fn(self, function);
    }

    fn visit_trait_item_fn(&mut self, function: &'ast TraitItemFn) {
        self.helper(Helper::Function, &function.sig.ident);
        visit::visit_trait_item_fn(self, function);
    }

    fn visit_item_struct(&mut self, item: &'ast ItemStruct) {
        self.row(&item.ident);
    }

    fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
        self.row(&item.ident);
    }

    fn visit_item_union(&mut self, item: &'ast ItemUnion) {
        self.helper(Helper::Type, &item.ident);
    }

    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        self.helper(Helper::Type, &item.ident);
        visit::visit_item_trait(self, item);
    }

    fn visit_item_type(&mut self, item: &'ast ItemType) {
        self.helper(Helper::Type, &item.ident);
    }

    fn visit_macro(&mut self, invocation: &'ast Macro) {
        self.written(invocation.tokens.clone());
    }
}

impl<'file> Style<'file> for TestShape {
    const RULE: Rule = Rule::TestShape;
    const READS: Reads = Reads::Every;

    fn new(source: &'file Source) -> TestShape {
        let tested = (source.place() == Place::Tests).then_some(());
        TestShape { found: Vec::new(), tested, rows: BTreeSet::new() }
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
