use syn::{FnArg, Ident, Pat, Path};

use super::struct_fields::StructFields;
use lint_core::Name;

pub struct Parameter {
    name: Name,
    type_name: Option<Name>,
}

impl Parameter {
    pub fn describe(input: &FnArg) -> Option<Parameter> {
        match input {
            FnArg::Receiver(receiver) => {
                Some(Parameter { name: Name::from(&Ident::from(receiver.self_token)), type_name: None })
            }
            FnArg::Typed(typed) => match &*typed.pat {
                Pat::Ident(ident) => {
                    Some(Parameter { name: Name::from(&ident.ident), type_name: StructFields::type_name(&typed.ty) })
                }
                _ => None,
            },
        }
    }

    pub fn named_by<'params>(params: &'params [Parameter], path: &Path) -> Option<&'params Parameter> {
        let ident = path.get_ident()?;
        params.iter().find(|parameter| parameter.name == *ident)
    }

    pub fn type_name(&self) -> Option<&Name> {
        self.type_name.as_ref()
    }
}
