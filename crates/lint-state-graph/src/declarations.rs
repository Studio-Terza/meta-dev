use lint_core::SourceFile;
mod vertex;

use std::collections::{BTreeMap, BTreeSet};

use algebra::{Monoid, Semigroup};
use syntax::{ItemImpl, ItemMod, ItemType};

use lint_core::Parsed;
use lint_core::TypeName;
use lint_sweep::{Contribute, Contributed, Contributing, Descend, Facts, Gathers, Kind};
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
pub struct DeclarationScan;

#[derive(Debug)]
pub struct DeclarationGathering {
    reading: Contributing<DeclarationScan>,
    shipped: Option<()>,
}

impl From<Parsed<'_, '_>> for DeclarationGathering {
    fn from(parsed: Parsed<'_, '_>) -> DeclarationGathering {
        DeclarationGathering {
            reading: Contributing::new(DeclarationScan),
            shipped: <&SourceFile>::from(parsed).shipped().map(|_| ()),
        }
    }
}

impl Gathers<Option<Declarations>> for DeclarationGathering {
    fn gathered(self, file: &SourceFile) -> Option<Declarations> {
        self.shipped.map(|()| {
            let reading = self.reading.concluded(file, |_, (), reading| reading);
            Declarations { vertices: reading.vertices, aliases: reading.aliases }
        })
    }
}

impl Semigroup for DeclarationReading {
    fn combine(mut self, other: DeclarationReading) -> DeclarationReading {
        self.vertices.extend(other.vertices);
        self.aliases.extend(other.aliases);
        self
    }
}

impl Monoid for DeclarationReading {}

impl FromIterator<Option<Declarations>> for Declarations {
    fn from_iter<Parts: IntoIterator<Item = Option<Declarations>>>(parts: Parts) -> Declarations {
        let gathered: DeclarationReading = Monoid::concat(
            parts
                .into_iter()
                .flatten()
                .map(|part| DeclarationReading { vertices: part.vertices, aliases: part.aliases }),
        );
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

impl Contribute for DeclarationScan {
    type Open = ();
    type Part = DeclarationReading;

    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemImpl, Kind::ItemType, Kind::ItemMod]
    }

    fn on_item_impl(
        &self,
        item: &ItemImpl,
        facts: &Facts<'_, ItemImpl>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), DeclarationReading> {
        let descend = Descend::from(facts.gate());
        let vertices = descend.entered().and_then(|_| Declarations::vertex(item)).into_iter().collect();
        Contributed::new((), DeclarationReading { vertices, aliases: BTreeMap::new() }, descend)
    }

    fn on_item_type(
        &self,
        item: &ItemType,
        _: &Facts<'_, ItemType>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), DeclarationReading> {
        let aliases = BTreeMap::from([(TypeName::named(&item.ident), TypeName::from_type(&item.ty))]);
        Contributed::new((), DeclarationReading { vertices: BTreeSet::new(), aliases }, Descend::Over)
    }

    fn on_item_mod(
        &self,
        _: &ItemMod,
        facts: &Facts<'_, ItemMod>,
        _around: &lint_sweep::Around<'_, Self::Open>,
    ) -> Contributed<(), DeclarationReading> {
        Contributed::descending(Descend::from(facts.gate()))
    }
}
