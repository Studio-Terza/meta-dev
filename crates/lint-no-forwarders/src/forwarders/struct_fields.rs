use std::collections::HashMap;

use syn::{Field, GenericArgument, GenericParam, Ident, Index, ItemStruct, Member, PathArguments, PathSegment, Type};

use lint_layout::Name;

pub struct StructFields<'file> {
    generics: Vec<Name>,
    fields: HashMap<FieldKey<'file>, Name>,
}

#[derive(PartialEq, Eq, Hash)]
enum FieldKey<'file> {
    Named(&'file Ident),
    Unnamed(Index),
}

impl<'file> StructFields<'file> {
    pub fn describe(item: &'file ItemStruct) -> StructFields<'file> {
        StructFields {
            generics: item
                .generics
                .params
                .iter()
                .filter_map(|param| match param {
                    GenericParam::Type(parameter) => Some(Name::from(&parameter.ident)),
                    _ => None,
                })
                .collect(),
            fields: item
                .fields
                .iter()
                .enumerate()
                .filter_map(|(index, field)| {
                    let key = Self::key_at(Index::from(index), field);
                    Self::type_name(&field.ty).map(|name| (key, name))
                })
                .collect(),
        }
    }

    pub fn field_type<'name>(&'name self, member: &'file Member, concrete: &'name [Name]) -> Option<&'name Name> {
        let key = match member {
            Member::Named(ident) => FieldKey::Named(ident),
            Member::Unnamed(index) => FieldKey::Unnamed(Index { index: index.index, span: index.span }),
        };
        let declared = self.fields.get(&key)?;
        match self.generics.iter().position(|generic| generic == declared) {
            Some(position) => concrete.get(position),
            None => Some(declared),
        }
    }

    pub fn type_name(ty: &Type) -> Option<Name> {
        match ty {
            Type::Reference(reference) => Self::type_name(&reference.elem),
            Type::Path(path) => path.path.segments.last().map(|segment| Name::from(&segment.ident)),
            _ => None,
        }
    }

    pub fn type_arguments(segment: &PathSegment) -> Vec<Name> {
        match &segment.arguments {
            PathArguments::AngleBracketed(arguments) => arguments.args.iter().filter_map(Self::argument_type).collect(),
            _ => Vec::new(),
        }
    }

    fn argument_type(argument: &GenericArgument) -> Option<Name> {
        match argument {
            GenericArgument::Type(ty) => Self::type_name(ty),
            _ => None,
        }
    }

    fn key_at(index: Index, field: &'file Field) -> FieldKey<'file> {
        field.ident.as_ref().map_or(FieldKey::Unnamed(index), FieldKey::Named)
    }
}
