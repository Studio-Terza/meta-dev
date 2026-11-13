use std::collections::HashMap;

use syn::{Field, GenericArgument, Index, ItemStruct, Member, PathArguments, PathSegment, Type};

use lint_core::Syntax;
use lint_layout::Name;

#[derive(Debug)]
pub struct StructFields {
    generics: Vec<Name>,
    fields: HashMap<FieldKey, Name>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum FieldKey {
    Named(Name),
    Unnamed(Index),
}

impl StructFields {
    pub fn describe(item: &ItemStruct) -> StructFields {
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

    pub fn field_type<'name>(&'name self, member: &Member, concrete: &'name [Name]) -> Option<&'name Name> {
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

    fn key_at(index: Index, field: &Field) -> FieldKey {
        field.ident.as_ref().map_or(FieldKey::Unnamed(index), |ident| FieldKey::Named(Name::from(ident)))
    }
}

impl From<&Member> for FieldKey {
    fn from(member: &Member) -> FieldKey {
        match member {
            Member::Named(ident) => FieldKey::Named(Name::from(ident)),
            Member::Unnamed(index) => FieldKey::Unnamed(Index { index: index.index, span: index.span }),
        }
    }
}
