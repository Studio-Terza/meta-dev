use std::collections::HashMap;

use syn::{Field, GenericArgument, GenericParam, Index, ItemStruct, Member, PathArguments, PathSegment, Type};

use lint_layout::Name;

pub struct StructFields {
    generics: Vec<Name>,
    fields: HashMap<Member, Name>,
}

impl StructFields {
    pub fn describe(item: &ItemStruct) -> StructFields {
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
                    let key = Self::key_at_index(index, field);
                    Self::type_name(&field.ty).map(|name| (key, name))
                })
                .collect(),
        }
    }

    pub fn field_type(&self, member: &Member, concrete: &[Name]) -> Option<Name> {
        let declared = self.fields.get(member)?;
        match self.generics.iter().position(|generic| generic == declared) {
            Some(position) => concrete.get(position).cloned(),
            None => Some(declared.clone()),
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

    fn key_at_index(index: usize, field: &Field) -> Member {
        field.ident.clone().map_or_else(|| Member::Unnamed(Index::from(index)), Member::Named)
    }
}
