mod vertex;

use std::collections::{BTreeMap, BTreeSet};
use syn::{Fields, ItemEnum, ItemImpl, ItemMod, ItemType};

use lint_core::Home;
use lint_core::Parsed;
use lint_core::TypeHomes;
use lint_core::TypeName;
use lint_core::Walk;
use lint_layout::Name;
use lint_sweep::{Descend, Gathers, Kind, Watch, Watching};
pub use vertex::Vertex;

#[derive(Debug)]
pub struct Declarations<'file> {
    vertices: BTreeSet<TypeName>,
    aliases: BTreeMap<TypeName, TypeName>,
    homes: TypeHomes<'file>,
    sum_types: BTreeMap<Home<'file>, BTreeSet<Name>>,
    reading: BTreeSet<Name>,
}

#[derive(Debug, Default)]
pub struct DeclarationReading {
    vertices: BTreeSet<TypeName>,
    aliases: BTreeMap<TypeName, TypeName>,
    reading: BTreeSet<Name>,
}

#[derive(Debug)]
pub struct DeclarationGathering<'head> {
    reading: DeclarationReading,
    homes: TypeHomes<'head>,
    shipped: Option<Home<'head>>,
}

impl<'head> From<Parsed<'_, 'head>> for DeclarationGathering<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> DeclarationGathering<'head> {
        DeclarationGathering {
            reading: DeclarationReading::default(),
            homes: TypeHomes::from(parsed),
            shipped: parsed.file().shipped().map(|_| Home::from(parsed.head())),
        }
    }
}

impl<'head> Gathers<Option<Declarations<'head>>> for DeclarationGathering<'head> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        let all: Vec<&mut Watching<'_>> = vec![&mut self.reading, &mut self.homes];
        self.shipped.map_or_default(|_| all)
    }

    fn gathered(self) -> Option<Declarations<'head>> {
        let reading = self.reading;
        let homes = self.homes;
        self.shipped.map(|home| {
            Declarations {
                vertices: reading.vertices,
                aliases: reading.aliases,
                homes,
                sum_types: BTreeMap::new(),
                reading: reading.reading,
            }
            .filed(home)
        })
    }
}

#[derive(Default)]
struct Gathered<'head> {
    vertices: BTreeSet<TypeName>,
    aliases: BTreeMap<TypeName, TypeName>,
    homes: Vec<TypeHomes<'head>>,
    sum_types: BTreeMap<Home<'head>, BTreeSet<Name>>,
}

impl<'head> Gathered<'head> {
    fn including(mut self, part: Declarations<'head>) -> Gathered<'head> {
        self.vertices.extend(part.vertices);
        self.aliases.extend(part.aliases);
        self.homes.push(part.homes);
        part.sum_types.into_iter().for_each(|(home, names)| self.sum_types.entry(home).or_default().extend(names));
        self
    }
}

impl<'head> FromIterator<Option<Declarations<'head>>> for Declarations<'head> {
    fn from_iter<Parts: IntoIterator<Item = Option<Declarations<'head>>>>(parts: Parts) -> Declarations<'head> {
        let gathered = parts.into_iter().flatten().fold(Gathered::default(), Gathered::including);
        let merged = Declarations {
            vertices: gathered.vertices,
            aliases: gathered.aliases,
            homes: gathered.homes.into_iter().collect(),
            sum_types: gathered.sum_types,
            reading: BTreeSet::new(),
        };
        merged.with_resolved_vertices()
    }
}

impl<'file> Declarations<'file> {
    pub fn resolved<'name>(&'name self, name: &'name TypeName) -> &'name TypeName {
        self.aliases.get(name).unwrap_or(name)
    }

    pub fn vertex_named(&self, name: &TypeName) -> Option<Vertex<'_>> {
        self.vertices.get(name).map(Vertex::new)
    }

    pub fn foreign_sum_type(&self, enum_name: &Name, from: &Home<'file>) -> Option<&Name> {
        let home = self.homes.home_of(enum_name, *from);
        self.sum_types.get(&home).and_then(|names| names.get(enum_name)).filter(|_| home != *from)
    }

    fn filed(mut self, home: Home<'file>) -> Declarations<'file> {
        let enums = std::mem::take(&mut self.reading);
        self.sum_types.entry(home).or_default().extend(enums);
        self
    }

    fn with_resolved_vertices(mut self) -> Declarations<'file> {
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

    fn sum_type(item: &ItemEnum) -> Option<&ItemEnum> {
        item.variants
            .iter()
            .find(|variant| matches!(variant.fields, Fields::Named(_) | Fields::Unnamed(_)))
            .map(|_| item)
    }
}

impl<'ast> Watch<'ast> for DeclarationReading {
    fn kinds(&self) -> &'static [Kind] {
        &[Kind::ItemImpl, Kind::ItemType, Kind::ItemEnum, Kind::ItemMod]
    }

    fn on_item_impl(&mut self, item: &'ast ItemImpl) -> Descend {
        let descend = Descend::from(Walk::outside_tests(&item.attrs));
        self.vertices.extend(descend.entered().and_then(|_| Declarations::vertex(item)));
        descend
    }

    fn on_item_type(&mut self, item: &'ast ItemType) -> Descend {
        self.aliases.insert(TypeName::named(&item.ident), TypeName::from_type(&item.ty));
        Descend::Over
    }

    fn on_item_enum(&mut self, item: &'ast ItemEnum) -> Descend {
        let entered = Descend::from(Walk::outside_tests(&item.attrs)).entered();
        self.reading.extend(entered.and_then(|_| Declarations::sum_type(item)).map(|found| Name::from(&found.ident)));
        Descend::Over
    }

    fn on_item_mod(&mut self, module: &'ast ItemMod) -> Descend {
        Descend::from(Walk::outside_tests(&module.attrs))
    }
}
