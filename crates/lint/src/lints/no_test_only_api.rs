use std::sync::Arc;

use proc_macro2::{TokenStream, TokenTree};
use syn::{Attribute, Field, ImplItem, Item, ItemImpl, ItemMod, Lit, Macro, Meta, Visibility};

use crate::finding::{Finding, Found, Rule};
use crate::gate::Gate;
use crate::graph::Graph;
use crate::source::{Place, Source};
use crate::walk::{Descent, Expanded, Hooks};
use crate::workspace::Package;

#[derive(Debug)]
pub(crate) struct NoTestOnlyApi<'file> {
    source: &'file Source,
    found: Vec<Found>,
    gates: Vec<Gate>,
}

#[derive(Clone, Copy, Debug)]
enum Hidden {
    GatedOnTest,
    DocHidden,
}

impl Hidden {
    fn of(attributes: &[Attribute]) -> Option<Hidden> {
        let gated = Gate::of(attributes).entered().xor(Some(())).map(|()| Hidden::GatedOnTest);
        gated.or_else(|| attributes.iter().find_map(Hidden::hiding))
    }

    fn hiding(attribute: &Attribute) -> Option<Hidden> {
        let list = match &attribute.meta {
            Meta::List(list) if list.path.is_ident("doc") => Some(list),
            Meta::List(_) | Meta::Path(_) | Meta::NameValue(_) => None,
        }?;
        let hidden =
            list.tokens.clone().into_iter().find(|tree| matches!(tree, TokenTree::Ident(ident) if ident == "hidden"));
        hidden.map(|_| Hidden::DocHidden)
    }

    fn found(self, visibility: &Visibility) -> Option<Found> {
        let (problem, fix) = match self {
            Hidden::GatedOnTest => {
                ("visible item gated on test", "remove the gate and test through the real public API")
            }
            Hidden::DocHidden => ("#[doc(hidden)] on a visible item", "document it as real API, or make it private"),
        };
        match visibility {
            Visibility::Public(token) => Some(Found::spanned(token.span, problem, fix)),
            Visibility::Restricted(restricted) => Some(Found::spanned(restricted.pub_token.span, problem, fix)),
            Visibility::Inherited => None,
        }
    }
}

impl NoTestOnlyApi<'_> {
    const TEST_WORDS: [&'static str; 5] = ["test", "tests", "testing", "mock", "mocks"];
    const CONDITIONS: [&'static str; 2] = ["cfg", "cfg_attr"];
    const FEATURE: &'static str = "feature";
    const FEATURE_FIX: &'static str = "remove the feature and test through the API every build has";

    fn testing(feature: &str) -> Option<()> {
        let letters: Vec<char> = feature.chars().collect();
        letters
            .chunk_by(|left, right| left.is_alphanumeric() == right.is_alphanumeric())
            .any(|word| {
                let spelled: String = word.iter().collect();
                NoTestOnlyApi::TEST_WORDS.contains(&spelled.as_str())
            })
            .then_some(())
    }

    fn features(tokens: TokenStream) -> Vec<Found> {
        let trees: Vec<TokenTree> = tokens.into_iter().collect();
        let keyed = trees.iter().zip(trees.iter().skip(2)).filter_map(|pair| match pair {
            (TokenTree::Ident(key), TokenTree::Literal(value)) if key == NoTestOnlyApi::FEATURE => {
                match Lit::new(value.clone()) {
                    Lit::Str(feature) => {
                        let name = feature.value();
                        NoTestOnlyApi::testing(&name).map(|()| {
                            Found::spanned(key.span(), format_args!("test feature {name}"), NoTestOnlyApi::FEATURE_FIX)
                        })
                    }
                    _ => None,
                }
            }
            _ => None,
        });
        let nested = trees.iter().filter_map(|tree| match tree {
            TokenTree::Group(group) => Some(NoTestOnlyApi::features(group.stream())),
            TokenTree::Ident(_) | TokenTree::Punct(_) | TokenTree::Literal(_) => None,
        });
        keyed.chain(nested.flatten()).collect()
    }

    fn shown(&self) -> Option<()> {
        let open = self.gates.iter().all(|gate| *gate == Gate::Entered);
        (open && self.source.place() == Place::Shipped).then_some(())
    }

    fn hidden(&mut self, attributes: &[Attribute], enclosing: Option<Hidden>, visibility: &Visibility) {
        let found = self
            .shown()
            .and_then(|()| Hidden::of(attributes).or(enclosing))
            .and_then(|hidden| hidden.found(visibility));
        self.found.extend(found);
    }
}

impl Hooks for NoTestOnlyApi<'_> {
    fn item(&mut self, item: &Item) -> Descent {
        let head = match item {
            Item::Const(constant) => Some((&constant.attrs, &constant.vis)),
            Item::Enum(enumeration) => Some((&enumeration.attrs, &enumeration.vis)),
            Item::Fn(function) => Some((&function.attrs, &function.vis)),
            Item::Mod(module) => Some((&module.attrs, &module.vis)),
            Item::Static(global) => Some((&global.attrs, &global.vis)),
            Item::Struct(structure) => Some((&structure.attrs, &structure.vis)),
            Item::Trait(trait_item) => Some((&trait_item.attrs, &trait_item.vis)),
            Item::Type(alias) => Some((&alias.attrs, &alias.vis)),
            Item::Union(union) => Some((&union.attrs, &union.vis)),
            Item::Use(import) => Some((&import.attrs, &import.vis)),
            _ => None,
        };
        head.map_or_default(|(attributes, visibility)| self.hidden(attributes, None, visibility));
        Descent::Into
    }

    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        self.gates.push(Gate::of(&module.attrs));
        Descent::Into
    }

    fn left_item_mod(&mut self, _: &ItemMod) {
        self.gates.pop();
    }

    fn item_impl(&mut self, item: &ItemImpl) -> Descent {
        let enclosing = Hidden::of(&item.attrs);
        item.items.iter().for_each(|member| {
            let head = match member {
                ImplItem::Const(constant) => Some((&constant.attrs, &constant.vis)),
                ImplItem::Fn(function) => Some((&function.attrs, &function.vis)),
                ImplItem::Type(alias) => Some((&alias.attrs, &alias.vis)),
                _ => None,
            };
            head.map_or_default(|(attributes, visibility)| self.hidden(attributes, enclosing, visibility));
        });
        Descent::Into
    }

    fn field(&mut self, field: &Field) -> Descent {
        self.hidden(&field.attrs, None, &field.vis);
        Descent::Into
    }

    fn attribute(&mut self, attribute: &Attribute) -> Descent {
        let condition = NoTestOnlyApi::CONDITIONS.iter().any(|name| attribute.path().is_ident(name));
        let found = match (&attribute.meta, condition) {
            (Meta::List(list), true) => NoTestOnlyApi::features(list.tokens.clone()),
            _ => Vec::new(),
        };
        self.found.extend(found);
        Descent::Into
    }

    fn macro_called(&mut self, invocation: &Macro) -> Expanded {
        let condition = NoTestOnlyApi::CONDITIONS.iter().any(|name| invocation.path.is_ident(name));
        let found = if condition { NoTestOnlyApi::features(invocation.tokens.clone()) } else { Vec::new() };
        self.found.extend(found);
        Expanded::Nothing
    }
}

impl<'file> Graph<'file> for NoTestOnlyApi<'file> {
    type Fact = Finding;

    fn new(source: &'file Source) -> NoTestOnlyApi<'file> {
        NoTestOnlyApi { source, found: Vec::new(), gates: Vec::new() }
    }

    fn facts(self) -> Vec<Finding> {
        let shown = self.source.shown();
        self.found.into_iter().map(|found| found.in_file(Rule::NoTestOnlyApi, shown)).collect()
    }

    fn reduce(facts: Vec<Finding>, packages: &[Arc<Package>]) -> Vec<Finding> {
        let manifests = packages.iter().flat_map(|package| {
            package.features().iter().filter_map(|feature| {
                NoTestOnlyApi::testing(feature.name()).map(|()| {
                    Found::at(
                        feature.line(),
                        format_args!("test feature {}", feature.name()),
                        NoTestOnlyApi::FEATURE_FIX,
                    )
                    .in_file(Rule::NoTestOnlyApi, package.manifest())
                })
            })
        });
        facts.into_iter().chain(manifests).collect()
    }
}
