use lint_core::SourceFile;
mod vertex;

use std::collections::{BTreeMap, BTreeSet};
use syntax::{ItemImpl, ItemMod, ItemType};

use lint_core::Parsed;
use lint_core::TypeName;
use lint_sweep::{Descend, Facts, Gathers, Kind, Watch, Watching};
pub use vertex::Vertex;

#[derive(Debug)]
pub struct Declarations {
    vertices: BTreeSet<TypeName>,
    aliases: BTreeMap<TypeName, TypeName>,
}

#[derive(Debug, Default)]
pub struct DeclarationReading {
    vertices: BTreeSet<TypeName>,
    aliases: BTreeMap<TypeName, TypeName>,
}

#[derive(Debug)]
pub struct DeclarationGathering {
    reading: DeclarationReading,
    shipped: Option<()>,
}

impl From<Parsed<'_, '_>> for DeclarationGathering {
    fn from(parsed: Parsed<'_, '_>) -> DeclarationGathering {
        DeclarationGathering {
            reading: DeclarationReading::default(),
            shipped: <&SourceFile>::from(parsed).shipped().map(|_| ()),
        }
    }
}

impl Gathers<Option<Declarations>> for DeclarationGathering {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        let all: Vec<&mut Watching<'_>> = vec![&mut self.reading];
        self.shipped.map_or_default(|()| all)
    }

    fn gathered(self) -> Option<Declarations> {
        let reading = self.reading;
        self.shipped.map(|()| Declarations { vertices: reading.vertices, aliases: reading.aliases })
    }
}

#[derive(Default)]
struct Gathered {
    vertices: BTreeSet<TypeName>,
    aliases: BTreeMap<TypeName, TypeName>,
}

impl Gathered {
    fn including(mut self, part: Declarations) -> Gathered {
        self.vertices.extend(part.vertices);
        self.aliases.extend(part.aliases);
        self
    }
}

impl FromIterator<Option<Declarations>> for Declarations {
    fn from_iter<Parts: IntoIterator<Item = Option<Declarations>>>(parts: Parts) -> Declarations {
        let gathered = parts.into_iter().flatten().fold(Gathered::default(), Gathered::including);
        let merged = Declarations { vertices: gathered.vertices, aliases: gathered.aliases };
        merged.with_resolved_vertices()
    }
}

impl Declarations {
    pub fn resolved<'name>(&'name self, name: &'name TypeName) -> &'name TypeName {
        self.aliases.get(name).unwrap_or(name)
    }

    pub fn vertex_named(&self, name: &TypeName) -> Option<Vertex<'_>> {
        self.vertices.get(name).map(Vertex::new)
    }

    fn with_resolved_vertices(mut self) -> Declarations {
        let vertices = std::mem::take(&mut self.vertices);
        self.vertices =
            vertices.into_iter().map(|vertex| self.aliases.get(&vertex).cloned().unwrap_or(vertex)).collect();
        self
    }

    fn vertex(item: &ItemImpl) -> Option<TypeName> {
        item.trait_
            .as_ref()
            .and_then(|(_, path, _)| path.segments.last())
            .filter(|segment| segment.ident == "State")
            .map(|_| TypeName::from_type(&item.self_ty))
    }
}

impl<'ast> Watch<'ast> for DeclarationReading {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemImpl, Kind::ItemType, Kind::ItemMod]
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl, facts: &Facts<'_, ItemImpl>) -> Descend {
        let descend = Descend::from(facts.gate());
        self.vertices.extend(descend.entered().and_then(|_| Declarations::vertex(item)));
        descend
    }

    fn on_item_type(&mut self, item: &'ast ItemType, _: &Facts<'_, ItemType>) -> Descend {
        self.aliases.insert(TypeName::named(&item.ident), TypeName::from_type(&item.ty));
        Descend::Over
    }

    fn on_item_mod(&mut self, _: &'ast ItemMod, facts: &Facts<'_, ItemMod>) -> Descend {
        Descend::from(facts.gate())
    }
}
