use std::fmt::{self, Write};

use syn::{ReturnType, Type};

use lint_layout::Name;

use crate::spelled::Spelled;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Described<'ast>(&'ast Type);

impl Described<'_> {
    pub(crate) fn name(found: &Type) -> Option<Name> {
        match found {
            Type::Path(_) => Spelled::last_of(found),
            Type::Reference(reference) => Self::name(&reference.elem),
            Type::Paren(paren) => Self::name(&paren.elem),
            Type::Group(group) => Self::name(&group.elem),
            other => Name::try_from(Described(other)).ok(),
        }
    }

    fn listed<Item: fmt::Display>(separator: Separator, items: impl Iterator<Item = Item>) -> Name {
        let texts: Vec<String> = items.map(|item| item.to_string()).collect();
        Name::from(texts.join(&separator.to_string()).as_str())
    }
}

#[derive(Clone, Copy)]
struct Returned<'ast>(&'ast ReturnType);

#[derive(Clone, Copy)]
enum Separator {
    Comma,
    Plus,
}

impl fmt::Display for Separator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Separator::Comma => formatter.write_str(", "),
            Separator::Plus => formatter.write_str(" + "),
        }
    }
}

impl fmt::Display for Returned<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ReturnType::Type(_, returned) => write!(formatter, " -> {}", Described(returned)),
            ReturnType::Default => Ok(()),
        }
    }
}

impl fmt::Display for Described<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Type::Slice(slice) => write!(formatter, "[{}]", Described(&slice.elem)),
            Type::Array(array) => write!(formatter, "[{}; _]", Described(&array.elem)),
            Type::Tuple(tuple) => {
                write!(formatter, "({})", Self::listed(Separator::Comma, tuple.elems.iter().map(Described)))
            }
            Type::TraitObject(object) => {
                write!(
                    formatter,
                    "dyn {}",
                    Self::listed(Separator::Plus, object.bounds.iter().filter_map(Spelled::bound_of))
                )
            }
            Type::BareFn(function) => write!(
                formatter,
                "fn({}){}",
                Self::listed(Separator::Comma, function.inputs.iter().map(|input| Described(&input.ty))),
                Returned(&function.output)
            ),
            Type::Ptr(pointer) => {
                write!(formatter, "*{} {}", pointer.mutability.map_or("const", |_| "mut"), Described(&pointer.elem))
            }
            Type::Reference(reference) => write!(formatter, "&{}", Described(&reference.elem)),
            Type::Paren(paren) => Described(&paren.elem).fmt(formatter),
            Type::Group(group) => Described(&group.elem).fmt(formatter),
            Type::Never(_) => formatter.write_str("!"),
            other => write!(formatter, "{}", Spelled::last_of(other).unwrap_or_else(|| Name::from("_"))),
        }
    }
}

impl TryFrom<Described<'_>> for Name {
    type Error = fmt::Error;

    fn try_from(found: Described<'_>) -> Result<Name, fmt::Error> {
        let mut text = String::new();
        write!(text, "{found}").map(|()| Name::from(text.as_str()))
    }
}
