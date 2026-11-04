use std::fmt::{self, Write};

use syn::{Type, TypeBareFn, TypeTraitObject, TypeTuple};

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

    fn listed<Item: fmt::Display>(
        formatter: &mut fmt::Formatter<'_>,
        separator: Separator,
        items: impl Iterator<Item = Item>,
    ) -> fmt::Result {
        items.enumerate().try_for_each(|(index, item)| {
            (index > 0)
                .then_some(separator)
                .map_or(Ok(()), |between| write!(formatter, "{between}"))
                .and_then(|()| write!(formatter, "{item}"))
        })
    }

    fn tupled(formatter: &mut fmt::Formatter<'_>, tuple: &TypeTuple) -> fmt::Result {
        formatter.write_str("(")?;
        Self::listed(formatter, Separator::Comma, tuple.elems.iter().map(Described))?;
        formatter.write_str(")")
    }

    fn object(formatter: &mut fmt::Formatter<'_>, object: &TypeTraitObject) -> fmt::Result {
        formatter.write_str("dyn ")?;
        let bounds = object.bounds.iter().filter_map(Spelled::bound_of);
        Self::listed(formatter, Separator::Plus, bounds)
    }

    fn function(formatter: &mut fmt::Formatter<'_>, function: &TypeBareFn) -> fmt::Result {
        formatter.write_str("fn(")?;
        Self::listed(formatter, Separator::Comma, function.inputs.iter().map(|input| Described(&input.ty)))?;
        formatter.write_str(")")?;
        Spelled::returned_by(&function.output)
            .map_or(Ok(()), |returned| write!(formatter, " -> {}", Described(returned)))
    }
}

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

impl fmt::Display for Described<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Type::Slice(slice) => write!(formatter, "[{}]", Described(&slice.elem)),
            Type::Array(array) => write!(formatter, "[{}; _]", Described(&array.elem)),
            Type::Tuple(tuple) => Self::tupled(formatter, tuple),
            Type::TraitObject(object) => Self::object(formatter, object),
            Type::BareFn(function) => Self::function(formatter, function),
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
