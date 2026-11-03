use syn::punctuated::Punctuated;
use syn::{Expr, FnArg, Ident, Path, Token};

use lint_core::Syntax;

use super::struct_fields::StructFields;
use lint_layout::Name;

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
            FnArg::Typed(typed) => Syntax::pat_ident(&typed.pat).map(|ident| Parameter {
                name: Name::from(&ident.ident),
                type_name: StructFields::type_name(&typed.ty),
            }),
        }
    }

    pub fn named_by<'params>(params: &'params [Parameter], path: &Path) -> Option<&'params Parameter> {
        let ident = path.get_ident()?;
        params.iter().find(|parameter| parameter.name == *ident)
    }

    pub fn forwarded<'expr>(expr: &'expr Expr, params: &[Parameter]) -> Option<&'expr Expr> {
        match expr {
            Expr::Path(path) => Parameter::named_by(params, &path.path).map(|_| expr),
            Expr::Field(field) => Self::forwarded(&field.base, params),
            Expr::Reference(reference) => Self::forwarded(&reference.expr, params),
            _ => None,
        }
    }

    pub fn forwarding(args: &Punctuated<Expr, Token![,]>, params: &[Parameter]) -> Option<()> {
        args.iter().all(|arg| Self::forwarded(arg, params).is_some()).then_some(())
    }

    pub fn type_name(&self) -> Option<&Name> {
        self.type_name.as_ref()
    }
}
