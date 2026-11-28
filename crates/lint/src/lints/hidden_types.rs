use std::collections::BTreeMap;

use proc_macro2::Ident;
use syn::{ExprMethodCall, ItemMod, ItemType, ItemUse, Macro, Path};

use crate::finding::{Findings, Found};
use crate::hooks::{Descent, Expanded, Hooks};
use crate::lint::{Lint, Reads};
use crate::source::Source;
use crate::use_leaf::UseLeaf;

#[derive(Debug)]
pub(crate) struct HiddenTypes {
    found: Vec<Found>,
    imported: BTreeMap<String, String>,
    bounded: Vec<Option<()>>,
}

impl HiddenTypes {
    const ERASING: [&'static str; 6] = ["Any", "TypeId", "type_id", "downcast", "downcast_ref", "downcast_mut"];
    const UNTYPED: [(&'static str, &'static str); 8] = [
        ("serde_json", "Value"),
        ("serde_json", "Map"),
        ("serde_json", "Number"),
        ("toml", "Value"),
        ("toml", "Table"),
        ("toml", "DeValue"),
        ("toml", "DeTable"),
        ("toml", "DeArray"),
    ];
    const BOUNDARY: &'static str = "boundary";

    fn erased(name: &Ident) -> Option<Found> {
        HiddenTypes::ERASING.iter().any(|erasing| name == erasing).then(|| {
            Found::spanned(
                name.span(),
                format_args!("{name} erases a type"),
                "take a generic or name an enum of the known types",
            )
        })
    }

    fn untyped<'path>(&self, home: Option<&str>, names: impl IntoIterator<Item = &'path Ident>) -> Vec<Found> {
        let outside = self.bounded.iter().flatten().next().copied().xor(Some(()));
        let untyped = names.into_iter().filter(|name| {
            let resolved = home.or_else(|| self.imported.get(&name.to_string()).map(String::as_str));
            HiddenTypes::UNTYPED
                .iter()
                .any(|(crate_name, type_name)| resolved == Some(*crate_name) && *name == type_name)
        });
        let found = untyped.map(|name| {
            Found::spanned(
                name.span(),
                format_args!("{name} is untyped data outside a boundary module"),
                "parse it into typed structs inside a module named boundary",
            )
        });
        outside.map_or_default(|()| found.collect())
    }
}

impl Hooks for HiddenTypes {
    fn path(&mut self, path: &Path) -> Descent {
        let names: Vec<&Ident> = path.segments.iter().map(|segment| &segment.ident).collect();
        let (first, rest) = names.split_first().map_or((None, &[][..]), |(first, rest)| (Some(*first), rest));
        let home = first.map(ToString::to_string);
        let erased = names.iter().filter_map(|name| HiddenTypes::erased(name));
        let untyped = self.untyped(None, first).into_iter().chain(self.untyped(home.as_deref(), rest.iter().copied()));
        let found: Vec<Found> = erased.chain(untyped).collect();
        self.found.extend(found);
        Descent::Into
    }

    fn item_use(&mut self, item: &ItemUse) -> Descent {
        let leaves = UseLeaf::of_tree(&item.tree);
        let homed: Vec<(Option<String>, &UseLeaf<'_>)> =
            leaves.iter().map(|leaf| (leaf.path().first().map(ToString::to_string), leaf)).collect();
        let found: Vec<Found> =
            homed.iter().flat_map(|(home, leaf)| self.untyped(home.as_deref(), leaf.end().original())).collect();
        let imported = homed.into_iter().filter_map(|(home, leaf)| leaf.end().bound().map(Ident::to_string).zip(home));
        self.imported.extend(imported);
        self.found.extend(found);
        Descent::Over
    }

    fn expr_method_call(&mut self, call: &ExprMethodCall) -> Descent {
        self.found.extend(HiddenTypes::erased(&call.method));
        Descent::Into
    }

    fn item_mod(&mut self, module: &ItemMod) -> Descent {
        self.bounded.push((module.ident == HiddenTypes::BOUNDARY).then_some(()));
        Descent::Into
    }

    fn left_item_mod(&mut self, _: &ItemMod) {
        self.bounded.pop();
    }

    fn item_type(&mut self, alias: &ItemType) -> Descent {
        let name = &alias.ident;
        self.found.extend([Found::spanned(
            name.span(),
            format_args!("type {name} renames a type"),
            "name the type itself, or make the alias a newtype",
        )]);
        Descent::Into
    }

    fn macro_called(&mut self, _: &Macro) -> Expanded {
        Expanded::Statements
    }
}

impl<'file> Lint<'file> for HiddenTypes {
    const HEADLINE: &'static str = "no hidden types: a type stays in the compiler's view, never erased to Any, renamed by an alias or carried as untyped data";
    const READS: Reads = Reads::Every;

    type Fact = Findings;

    fn new(source: &'file Source) -> HiddenTypes {
        let bounded = source.module().names().iter().any(|name| name == HiddenTypes::BOUNDARY).then_some(());
        HiddenTypes { found: Vec::new(), imported: BTreeMap::new(), bounded: vec![bounded] }
    }

    fn fact(self, source: &'file Source) -> Findings {
        Findings::found(HiddenTypes::HEADLINE, source.shown(), self.found)
    }
}
