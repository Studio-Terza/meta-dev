use std::collections::BTreeSet;

use proc_macro2::{Delimiter, Ident, Punct, Spacing, TokenStream, TokenTree};
use syn::visit::{self, Visit};
use syn::{
    Attribute, ImplItemFn, ItemConst, ItemEnum, ItemFn, ItemMod, ItemStatic, ItemStruct, ItemTrait, ItemType,
    ItemUnion, Lit, Macro, Meta, TraitItemFn, TypePath,
};

use crate::finding::{Found, Rule};
use crate::gate::{Gate, TestMark};
use crate::source::{Place, Source};
use crate::style::{Reads, Style};
use crate::terms::Fix;
use crate::walk::{Descent, Expanded, Hooks};

#[derive(Debug)]
pub(crate) struct TestShape {
    found: Vec<Found>,
    tested: Option<()>,
    gated: Vec<Option<()>>,
    holdings: Vec<Option<Holding>>,
    tables: Named,
    shapes: Vec<Shape>,
}

#[derive(Debug)]
struct Holding {
    name: Ident,
    kind: Held,
    flagged: Option<()>,
}

#[derive(Clone, Copy, Debug)]
enum Held {
    Test,
    Constant,
}

#[derive(Debug, Default)]
struct Named(BTreeSet<Ident>);

#[derive(Debug)]
struct Shape {
    name: Ident,
    named: Named,
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
    const DERIVE: &'static str = "derive";
    const FIXTURE: &'static str = "Fixture";
    const MARK: char = '#';
    const TRUTHS: [&'static str; 2] = ["true", "false"];

    fn testing(&self) -> Option<()> {
        self.tested.or(self.gated.iter().flatten().next().copied())
    }

    fn helped(&self, kind: Helper, name: &Ident) -> Option<Found> {
        let problem = match kind {
            Helper::Function => format!("test helper fn {name}"),
            Helper::Type => format!("test helper type {name}"),
        };
        self.testing().map(|()| Found::spanned(name.span(), problem, TestShape::HELPER_FIX))
    }

    fn helper(&mut self, kind: Helper, name: &Ident) {
        let found = self.helped(kind, name);
        self.found.extend(found);
    }

    fn function(&mut self, attributes: &[Attribute], name: &Ident) -> Descent {
        let holding =
            TestMark::among(attributes).map(|_| Holding { name: name.clone(), kind: Held::Test, flagged: None });
        let helper = holding.is_none().then(|| self.helped(Helper::Function, name)).flatten();
        self.found.extend(helper);
        self.holdings.push(holding);
        Descent::Into
    }

    fn held(&mut self, name: &Ident) -> Descent {
        let holding = self.testing().map(|()| Holding { name: name.clone(), kind: Held::Constant, flagged: None });
        self.holdings.push(holding);
        Descent::Into
    }

    fn released(&mut self) {
        self.holdings.pop();
    }

    fn literal(&mut self) {
        let unflagged = self.holdings.last_mut().and_then(Option::as_mut).filter(|holding| holding.flagged.is_none());
        let found = unflagged.map(|holding| {
            holding.flagged.replace(());
            let name = &holding.name;
            let problem = match holding.kind {
                Held::Test => format!("literal in test {name}"),
                Held::Constant => format!("literal in test constant {name}"),
            };
            Found::spanned(name.span(), problem, TestShape::LITERAL_FIX)
        });
        self.found.extend(found);
    }

    fn named_literal(&mut self, name: &Ident) {
        self.found.extend(Some(Found::spanned(
            name.span(),
            format_args!("literal in test {name}"),
            TestShape::LITERAL_FIX,
        )));
    }

    fn tokens(stream: TokenStream) -> Option<()> {
        stream.into_iter().find_map(|tree| match tree {
            TokenTree::Literal(_) => Some(()),
            TokenTree::Ident(ident) => TestShape::TRUTHS.iter().find(|truth| ident == *truth).map(|_| ()),
            TokenTree::Group(group) => TestShape::tokens(group.stream()),
            TokenTree::Punct(_) => None,
        })
    }

    fn written(&mut self, tokens: TokenStream) -> Scan {
        tokens.into_iter().fold(Scan::Idle, |scanned, tree| self.scanned(scanned, tree))
    }

    fn scanned(&mut self, scan: Scan, tree: TokenTree) -> Scan {
        match (scan, tree) {
            (Scan::Named(name), TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                TestShape::tokens(group.stream()).map_or_default(|()| self.named_literal(&name));
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

    fn rows(&self) -> BTreeSet<Ident> {
        TestShape::grown(self.tables.0.clone(), &self.shapes)
    }

    fn grown(rows: BTreeSet<Ident>, shapes: &[Shape]) -> BTreeSet<Ident> {
        let reached: BTreeSet<Ident> = shapes
            .iter()
            .filter(|shape| rows.contains(&shape.name))
            .flat_map(|shape| shape.named.0.iter().filter(|inner| !rows.contains(*inner)).cloned())
            .collect();
        match reached.first() {
            Some(_) => TestShape::grown(rows.into_iter().chain(reached).collect(), shapes),
            None => rows,
        }
    }

    fn shaped(&mut self, attributes: &[Attribute], name: &Ident, named: Named) -> Descent {
        let fixture = attributes.iter().find(|attribute| TestShape::derives_fixture(attribute).is_some());
        self.tables.0.extend(fixture.map(|_| name.clone()));
        self.testing().map_or_default(|()| self.shapes.push(Shape { name: name.clone(), named }));
        Descent::Over
    }

    fn derives_fixture(attribute: &Attribute) -> Option<()> {
        let derived = match &attribute.meta {
            Meta::List(list) if list.path.is_ident(TestShape::DERIVE) => Some(list.tokens.clone()),
            Meta::List(_) | Meta::Path(_) | Meta::NameValue(_) => None,
        }?;
        derived
            .into_iter()
            .find(|tree| matches!(tree, TokenTree::Ident(ident) if ident == TestShape::FIXTURE))
            .map(|_| ())
    }
}

impl<'ast> Visit<'ast> for Named {
    fn visit_type_path(&mut self, path: &'ast TypePath) {
        self.0.extend(path.path.segments.iter().map(|segment| segment.ident.clone()));
        visit::visit_type_path(self, path);
    }
}

impl Hooks for TestShape {
    fn left_file(&mut self, _: &syn::File) {
        let rows = self.rows();
        let helpers: Vec<Found> = self
            .shapes
            .iter()
            .filter(|shape| !rows.contains(&shape.name))
            .filter_map(|shape| self.helped(Helper::Type, &shape.name))
            .collect();
        self.found.extend(helpers);
    }

    fn macro_called(&mut self, invocation: &Macro) -> Expanded {
        let held = self.holdings.last().and_then(Option::as_ref).map(|_| ());
        held.and_then(|()| TestShape::tokens(invocation.tokens.clone())).map_or_default(|()| self.literal());
        held.xor(Some(())).and(self.testing()).map_or_default(|()| {
            self.written(invocation.tokens.clone());
        });
        Expanded::Nothing
    }

    fn lit(&mut self, _: &Lit) -> Descent {
        self.literal();
        Descent::Over
    }

    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        self.gated.push(Gate::of(&module.attrs).entered().xor(Some(())));
        Descent::Into
    }

    fn left_item_mod(&mut self, _: &ItemMod) {
        self.gated.pop();
    }

    fn item_fn(&mut self, function: &ItemFn) -> Descent {
        self.function(&function.attrs, &function.sig.ident)
    }

    fn left_item_fn(&mut self, _: &ItemFn) {
        self.released();
    }

    fn impl_item_fn(&mut self, function: &ImplItemFn) -> Descent {
        self.function(&function.attrs, &function.sig.ident)
    }

    fn left_impl_item_fn(&mut self, _: &ImplItemFn) {
        self.released();
    }

    fn trait_item_fn(&mut self, function: &TraitItemFn) -> Descent {
        self.helper(Helper::Function, &function.sig.ident);
        Descent::Over
    }

    fn item_struct(&mut self, item: &ItemStruct) -> Descent {
        let mut named = Named::default();
        named.visit_fields(&item.fields);
        self.shaped(&item.attrs, &item.ident, named)
    }

    fn item_enum(&mut self, item: &ItemEnum) -> Descent {
        let mut named = Named::default();
        item.variants.iter().for_each(|variant| named.visit_fields(&variant.fields));
        self.shaped(&item.attrs, &item.ident, named)
    }

    fn item_union(&mut self, item: &ItemUnion) -> Descent {
        self.helper(Helper::Type, &item.ident);
        Descent::Over
    }

    fn item_trait(&mut self, item: &ItemTrait) -> Descent {
        self.helper(Helper::Type, &item.ident);
        Descent::Over
    }

    fn item_type(&mut self, item: &ItemType) -> Descent {
        self.helper(Helper::Type, &item.ident);
        Descent::Over
    }

    fn item_const(&mut self, item: &ItemConst) -> Descent {
        self.tables.visit_type(&item.ty);
        self.held(&item.ident)
    }

    fn left_item_const(&mut self, _: &ItemConst) {
        self.released();
    }

    fn item_static(&mut self, item: &ItemStatic) -> Descent {
        self.tables.visit_type(&item.ty);
        self.held(&item.ident)
    }

    fn left_item_static(&mut self, _: &ItemStatic) {
        self.released();
    }
}

impl<'file> Style<'file> for TestShape {
    const RULE: Rule = Rule::TestShape;
    const READS: Reads = Reads::Every;

    fn new(source: &'file Source) -> TestShape {
        TestShape {
            found: Vec::new(),
            tested: (source.place() == Place::Tests).then_some(()),
            gated: Vec::new(),
            holdings: Vec::new(),
            tables: Named::default(),
            shapes: Vec::new(),
        }
    }

    fn found(self) -> Vec<Found> {
        self.found
    }
}
