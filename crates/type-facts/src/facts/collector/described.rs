use std::fmt::{self, Write};

use syn::Type;

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
}

struct Listed<Item> {
    separator: Separator,
    items: Vec<Item>,
}

impl<Item> Listed<Item> {
    fn new(separator: Separator, items: impl Iterator<Item = Item>) -> Self {
        Self { separator, items: items.collect() }
    }
}

#[derive(Clone, Copy)]
struct Returned<'ast>(Option<&'ast Type>);

#[derive(Clone, Copy)]
enum Separator {
    Comma,
    Plus,
}

impl fmt::Display for Separator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Comma => formatter.write_str(", "),
            Self::Plus => formatter.write_str(" + "),
        }
    }
}

impl<Item: fmt::Display> fmt::Display for Listed<Item> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.items.split_first().into_iter().try_for_each(|(first, rest)| {
            write!(formatter, "{first}")?;
            rest.iter().try_for_each(|item| write!(formatter, "{}{item}", self.separator))
        })
    }
}

impl fmt::Display for Returned<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(returned) => write!(formatter, " -> {}", Described(returned)),
            None => Ok(()),
        }
    }
}

impl fmt::Display for Described<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Type::Slice(slice) => write!(formatter, "[{}]", Described(&slice.elem)),
            Type::Array(array) => write!(formatter, "[{}; _]", Described(&array.elem)),
            Type::Tuple(tuple) => {
                write!(formatter, "({})", Listed::new(Separator::Comma, tuple.elems.iter().map(Described)))
            }
            Type::TraitObject(object) => {
                write!(
                    formatter,
                    "dyn {}",
                    Listed::new(Separator::Plus, object.bounds.iter().filter_map(Spelled::bound_of))
                )
            }
            Type::BareFn(function) => write!(
                formatter,
                "fn({}){}",
                Listed::new(Separator::Comma, function.inputs.iter().map(|input| Described(&input.ty))),
                Returned(Spelled::returned_by(&function.output))
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

    fn try_from(found: Described<'_>) -> Result<Self, fmt::Error> {
        let mut text = String::new();
        write!(text, "{found}").map(|()| Self::from(text.as_str()))
    }
}
