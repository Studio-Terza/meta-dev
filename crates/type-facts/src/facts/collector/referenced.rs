use std::path::Path;

use syn::visit::{self, Visit};
use syn::{Field, Generics, Ident, ItemTrait, PathSegment, TraitItem};

use lint_core::Site;
use lint_layout::{LineNumber, Name, Whole, Word, words};

use crate::dependency::Dependency;
use crate::via::Via;

pub(crate) struct Referenced<'item, 'file> {
    file: &'file Path,
    own: &'item Ident,
    parameters: Vec<&'item Ident>,
    via: Via,
    found: Vec<Dependency<'file>>,
}

impl<'item, 'file> Referenced<'item, 'file> {
    const ITSELF: [Word<'static>; 1] = words!["Self"];

    pub(crate) fn started(file: &'file Path, own: &'item Ident, generics: &'item Generics) -> Referenced<'item, 'file> {
        Referenced {
            file,
            own,
            parameters: generics.type_params().map(|parameter| &parameter.ident).collect(),
            via: Via::Bound,
            found: Vec::new(),
        }
    }

    pub(crate) fn of_fields(
        mut self,
        via: Via,
        fields: impl Iterator<Item = &'item Field>,
        generics: &Generics,
    ) -> Vec<Dependency<'file>> {
        self.via = via;
        fields.for_each(|field| self.visit_type(&field.ty));
        self.bounded(generics)
    }

    pub(crate) fn of_trait(mut self, found: &ItemTrait) -> Vec<Dependency<'file>> {
        found.supertraits.iter().for_each(|bound| self.visit_type_param_bound(bound));
        let associated = found.items.iter().filter_map(|item| match item {
            TraitItem::Type(associated) => Some(associated),
            _ => None,
        });
        associated.for_each(|associated| {
            associated.bounds.iter().for_each(|bound| self.visit_type_param_bound(bound));
            self.visit_generics(&associated.generics);
        });
        self.bounded(&found.generics)
    }

    fn bounded(mut self, generics: &Generics) -> Vec<Dependency<'file>> {
        self.via = Via::Bound;
        self.visit_generics(generics);
        self.found
    }

    fn dependency(&self, path: &syn::Path, segment: &PathSegment) -> Option<Dependency<'file>> {
        let qualifier = path.segments.first().filter(|_| path.segments.len() > 1).map(|first| Name::from(&first.ident));
        let named = Name::from(&segment.ident);
        let resolved = match named.matches::<Whole>(&Self::ITSELF) {
            Some(_) => Name::from(self.own),
            None => named,
        };
        let site = Site::new(self.file, LineNumber::from(segment.ident.span()));
        (!self.parameters.contains(&&segment.ident))
            .then(|| Dependency::new(resolved, site, self.via).qualified(qualifier))
    }

    fn visit_arguments(&mut self, segment: &PathSegment) {
        visit::visit_path_arguments(self, &segment.arguments);
    }
}

impl<'ast> Visit<'ast> for Referenced<'_, '_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let found = path.segments.last().and_then(|segment| self.dependency(path, segment));
        self.found.extend(found);
        path.segments.iter().for_each(|segment| self.visit_arguments(segment));
    }
}
