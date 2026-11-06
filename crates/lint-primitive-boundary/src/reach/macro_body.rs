use syn::visit::Visit;
use syn::{File, Item, Macro, Type};

pub enum MacroBody {
    Items(Vec<Item>),
    Type(Box<Type>),
    Opaque,
}

impl MacroBody {
    pub fn read(invocation: &Macro) -> MacroBody {
        invocation
            .parse_body::<File>()
            .map(|file| MacroBody::Items(file.items))
            .or_else(|_| invocation.parse_body::<Type>().map(|ty| MacroBody::Type(Box::new(ty))))
            .unwrap_or(MacroBody::Opaque)
    }

    pub fn walked(&self, visitor: &mut impl for<'ast> Visit<'ast>) {
        match self {
            MacroBody::Items(items) => items.iter().for_each(|item| visitor.visit_item(item)),
            MacroBody::Type(ty) => visitor.visit_type(ty),
            MacroBody::Opaque => {}
        }
    }
}
