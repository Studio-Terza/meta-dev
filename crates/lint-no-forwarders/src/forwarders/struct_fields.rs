use std::collections::HashMap;

use syn::{Field, GenericArgument, Ident, Index, ItemStruct, Member, PathArguments, PathSegment, Type};

use lint_core::Syntax;
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
    pub fn describe(item: &'file ItemStruct) -> Self {
        StructFields {
            generics: item
                .generics
                .params
                .iter()
                .filter_map(Syntax::type_param)
                .map(|parameter| Name::from(&parameter.ident))
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
        let declared = self.fields.get(&FieldKey::from(member))?;
        self.generics
            .iter()
            .position(|generic| generic == declared)
            .map_or(Some(declared), |position| concrete.get(position))
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

impl<'file> From<&'file Member> for FieldKey<'file> {
    fn from(member: &'file Member) -> Self {
        match member {
            Member::Named(ident) => FieldKey::Named(ident),
            Member::Unnamed(index) => FieldKey::Unnamed(Index { index: index.index, span: index.span }),
        }
    }
}
