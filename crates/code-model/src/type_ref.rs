use std::convert::Infallible;

use std::fmt;

use lint_layout::Name;

use crate::resolved::Resolved;
use crate::sited::{Sited, Spelling};

pub trait TypeUses {
    fn named(&mut self, named: &Named<Sited>);
}

pub(crate) trait Outline {
    type Kind;

    fn named(self, named: &Named<Sited>) -> Self::Kind;

    fn grouped(self, parts: &[TypeRef]) -> Self::Kind;

    fn sequence(self) -> Self::Kind;

    fn bounded(self, bounds: &[TypeRef]) -> Self::Kind;

    fn other(self) -> Self::Kind;
}

pub(crate) trait Rebuild<From> {
    type Into;
    type Fail;

    fn path(&mut self, path: &From) -> Self::Into;

    fn child(&mut self, ty: &TypeRef<From>) -> Result<TypeRef<Self::Into>, Self::Fail>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeRef<Bound = Sited> {
    Named(Named<Bound>),
    Tuple(Vec<TypeRef<Bound>>),
    Reference(Box<TypeRef<Bound>>),
    Pointer(Box<TypeRef<Bound>>),
    Slice(Box<TypeRef<Bound>>),
    Array(Arrayed<Bound>),
    Bounded(Vec<TypeRef<Bound>>),
    Projected(Projection<Bound>),
    Function(Callable<Bound>),
    Opaque,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Named<Bound> {
    path: Bound,
    arguments: Vec<TypeRef<Bound>>,
    bindings: Vec<TypeRef<Bound>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arrayed<Bound> {
    element: Box<TypeRef<Bound>>,
    measured: Vec<TypeRef<Bound>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection<Bound> {
    subject: Box<TypeRef<Bound>>,
    arguments: Vec<TypeRef<Bound>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Callable<Bound> {
    inputs: Vec<TypeRef<Bound>>,
    output: Box<TypeRef<Bound>>,
}

#[derive(Clone, Copy, Debug)]
struct Parts<'ty, Bound> {
    first: &'ty [TypeRef<Bound>],
    rest: &'ty [TypeRef<Bound>],
}

impl<'ty, Bound> Parts<'ty, Bound> {
    fn children(self) -> impl Iterator<Item = &'ty TypeRef<Bound>> {
        self.first.iter().chain(self.rest)
    }
}

impl<Bound> Named<Bound> {
    #[must_use]
    pub fn new(path: Bound, arguments: Vec<TypeRef<Bound>>) -> Named<Bound> {
        Named { path, arguments, bindings: Vec::new() }
    }

    #[must_use]
    pub fn bound(path: Bound, arguments: Vec<TypeRef<Bound>>, bindings: Vec<TypeRef<Bound>>) -> Named<Bound> {
        Named { path, arguments, bindings }
    }

    #[must_use]
    pub fn arguments(&self) -> &[TypeRef<Bound>] {
        &self.arguments
    }

    fn parts(&self) -> Parts<'_, Bound> {
        Parts { first: &self.arguments, rest: &self.bindings }
    }

    fn rebuilt<Rule: Rebuild<Bound>>(&self, rule: &mut Rule) -> Result<Named<Rule::Into>, Rule::Fail> {
        Ok(Named {
            path: rule.path(&self.path),
            arguments: TypeRef::each_rebuilt(&self.arguments, rule)?,
            bindings: TypeRef::each_rebuilt(&self.bindings, rule)?,
        })
    }
}

impl<Bound> AsRef<Bound> for Named<Bound> {
    fn as_ref(&self) -> &Bound {
        &self.path
    }
}

impl AsRef<Resolved> for Named<Sited> {
    fn as_ref(&self) -> &Resolved {
        self.path.as_ref()
    }
}

impl Named<Sited> {
    pub fn each_inner_use(&self, uses: &mut impl TypeUses) {
        self.parts().children().for_each(|ty| ty.each_use(uses));
    }

    fn last_spelled(&self) -> Option<&Name> {
        AsRef::<Spelling>::as_ref(&self.path).written()?.last()
    }
}

impl<Bound> Arrayed<Bound> {
    #[must_use]
    pub fn new(element: TypeRef<Bound>, measured: Vec<TypeRef<Bound>>) -> Arrayed<Bound> {
        Arrayed { element: Box::new(element), measured }
    }

    fn parts(&self) -> Parts<'_, Bound> {
        Parts { first: std::slice::from_ref(&*self.element), rest: &self.measured }
    }

    fn rebuilt<Rule: Rebuild<Bound>>(&self, rule: &mut Rule) -> Result<Arrayed<Rule::Into>, Rule::Fail> {
        Ok(Arrayed {
            element: Box::new(rule.child(&self.element)?),
            measured: TypeRef::each_rebuilt(&self.measured, rule)?,
        })
    }
}

impl<Bound> Projection<Bound> {
    #[must_use]
    pub fn new(subject: TypeRef<Bound>, arguments: Vec<TypeRef<Bound>>) -> Projection<Bound> {
        Projection { subject: Box::new(subject), arguments }
    }

    fn parts(&self) -> Parts<'_, Bound> {
        Parts { first: std::slice::from_ref(&*self.subject), rest: &self.arguments }
    }

    fn rebuilt<Rule: Rebuild<Bound>>(&self, rule: &mut Rule) -> Result<Projection<Rule::Into>, Rule::Fail> {
        Ok(Projection {
            subject: Box::new(rule.child(&self.subject)?),
            arguments: TypeRef::each_rebuilt(&self.arguments, rule)?,
        })
    }
}

impl<Bound> Callable<Bound> {
    #[must_use]
    pub fn new(inputs: Vec<TypeRef<Bound>>, output: TypeRef<Bound>) -> Callable<Bound> {
        Callable { inputs, output: Box::new(output) }
    }

    fn parts(&self) -> Parts<'_, Bound> {
        Parts { first: &self.inputs, rest: std::slice::from_ref(&*self.output) }
    }

    fn rebuilt<Rule: Rebuild<Bound>>(&self, rule: &mut Rule) -> Result<Callable<Rule::Into>, Rule::Fail> {
        Ok(Callable { inputs: TypeRef::each_rebuilt(&self.inputs, rule)?, output: Box::new(rule.child(&self.output)?) })
    }
}

impl<Bound> TypeRef<Bound> {
    pub fn mapped<Into>(&self, resolve: &mut impl FnMut(&Bound) -> Into) -> TypeRef<Into> {
        let Ok(mapped) = Mapping { resolve }.child(self);
        mapped
    }

    #[must_use]
    pub fn named(&self) -> Option<&Named<Bound>> {
        match self {
            TypeRef::Named(named) => Some(named),
            TypeRef::Tuple(_)
            | TypeRef::Reference(_)
            | TypeRef::Pointer(_)
            | TypeRef::Slice(_)
            | TypeRef::Array(_)
            | TypeRef::Bounded(_)
            | TypeRef::Projected(_)
            | TypeRef::Function(_)
            | TypeRef::Opaque => None,
        }
    }

    #[must_use]
    pub fn dereferenced(&self) -> &TypeRef<Bound> {
        match self {
            TypeRef::Reference(inner) => inner.dereferenced(),
            TypeRef::Named(_)
            | TypeRef::Tuple(_)
            | TypeRef::Pointer(_)
            | TypeRef::Slice(_)
            | TypeRef::Array(_)
            | TypeRef::Bounded(_)
            | TypeRef::Projected(_)
            | TypeRef::Function(_)
            | TypeRef::Opaque => self,
        }
    }

    #[must_use]
    pub fn unit(&self) -> Option<&TypeRef<Bound>> {
        match self {
            TypeRef::Tuple(elements) => elements.first().xor(Some(self)),
            TypeRef::Named(_)
            | TypeRef::Reference(_)
            | TypeRef::Pointer(_)
            | TypeRef::Slice(_)
            | TypeRef::Array(_)
            | TypeRef::Bounded(_)
            | TypeRef::Projected(_)
            | TypeRef::Function(_)
            | TypeRef::Opaque => None,
        }
    }

    #[must_use]
    pub fn respelled(self, respell: impl FnOnce(Bound) -> Bound) -> TypeRef<Bound> {
        match self {
            TypeRef::Named(named) => TypeRef::Named(Named {
                path: respell(named.path),
                arguments: named.arguments,
                bindings: named.bindings,
            }),
            other => other,
        }
    }

    pub(crate) fn rebuilt<Rule: Rebuild<Bound>>(&self, rule: &mut Rule) -> Result<TypeRef<Rule::Into>, Rule::Fail> {
        match self {
            TypeRef::Named(named) => named.rebuilt(rule).map(TypeRef::Named),
            TypeRef::Tuple(elements) => TypeRef::each_rebuilt(elements, rule).map(TypeRef::Tuple),
            TypeRef::Reference(inner) => rule.child(inner).map(Box::new).map(TypeRef::Reference),
            TypeRef::Pointer(inner) => rule.child(inner).map(Box::new).map(TypeRef::Pointer),
            TypeRef::Slice(inner) => rule.child(inner).map(Box::new).map(TypeRef::Slice),
            TypeRef::Array(arrayed) => arrayed.rebuilt(rule).map(TypeRef::Array),
            TypeRef::Bounded(bounds) => TypeRef::each_rebuilt(bounds, rule).map(TypeRef::Bounded),
            TypeRef::Projected(projection) => projection.rebuilt(rule).map(TypeRef::Projected),
            TypeRef::Function(callable) => callable.rebuilt(rule).map(TypeRef::Function),
            TypeRef::Opaque => Ok(TypeRef::Opaque),
        }
    }

    fn parts(&self) -> Parts<'_, Bound> {
        match self {
            TypeRef::Named(named) => named.parts(),
            TypeRef::Array(arrayed) => arrayed.parts(),
            TypeRef::Projected(projection) => projection.parts(),
            TypeRef::Function(callable) => callable.parts(),
            TypeRef::Tuple(elements) | TypeRef::Bounded(elements) => Parts { first: elements, rest: &[] },
            TypeRef::Reference(inner) | TypeRef::Pointer(inner) | TypeRef::Slice(inner) => {
                Parts { first: std::slice::from_ref(&**inner), rest: &[] }
            }
            TypeRef::Opaque => Parts { first: &[], rest: &[] },
        }
    }

    fn each_rebuilt<Rule: Rebuild<Bound>>(
        types: &[TypeRef<Bound>],
        rule: &mut Rule,
    ) -> Result<Vec<TypeRef<Rule::Into>>, Rule::Fail> {
        types.iter().map(|ty| rule.child(ty)).collect()
    }
}

impl TypeRef {
    pub fn each_use(&self, uses: &mut impl TypeUses) {
        match self.named() {
            Some(named) => uses.named(named),
            None => self.parts().children().for_each(|child| child.each_use(uses)),
        }
    }

    pub(crate) fn outlined<Shape: Outline>(&self, outline: Shape) -> Shape::Kind {
        match self {
            TypeRef::Reference(inner) => inner.outlined(outline),
            TypeRef::Named(named) => outline.named(named),
            TypeRef::Tuple(parts) => outline.grouped(parts),
            TypeRef::Slice(_) | TypeRef::Array(_) => outline.sequence(),
            TypeRef::Bounded(bounds) => outline.bounded(bounds),
            TypeRef::Pointer(_) | TypeRef::Projected(_) | TypeRef::Function(_) | TypeRef::Opaque => outline.other(),
        }
    }
}

#[derive(Debug)]
struct Mapping<'resolve, Resolve> {
    resolve: &'resolve mut Resolve,
}

impl<From, Into, Resolve: FnMut(&From) -> Into> Rebuild<From> for Mapping<'_, Resolve> {
    type Into = Into;
    type Fail = Infallible;

    fn path(&mut self, path: &From) -> Into {
        (self.resolve)(path)
    }

    fn child(&mut self, ty: &TypeRef<From>) -> Result<TypeRef<Into>, Infallible> {
        ty.rebuilt(self)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Brief<'ty>(&'ty TypeRef);

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
struct Listed<'ty> {
    separator: Separator,
    items: &'ty [TypeRef],
}

impl fmt::Display for Listed<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.items.split_first().into_iter().try_for_each(|(first, rest)| {
            write!(formatter, "{}", Brief(first))?;
            rest.iter().try_for_each(|item| write!(formatter, "{}{}", self.separator, Brief(item)))
        })
    }
}

impl fmt::Display for Brief<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            TypeRef::Named(named) => write!(formatter, "{}", named.last_spelled().unwrap_or(&Name::from("_"))),
            TypeRef::Tuple(items) => write!(formatter, "({})", Listed { separator: Separator::Comma, items }),
            TypeRef::Reference(inner) => write!(formatter, "&{}", Brief(inner)),
            TypeRef::Pointer(inner) => write!(formatter, "*const {}", Brief(inner)),
            TypeRef::Slice(inner) => write!(formatter, "[{}]", Brief(inner)),
            TypeRef::Array(arrayed) => write!(formatter, "[{}; _]", Brief(&arrayed.element)),
            TypeRef::Bounded(bounds) => {
                write!(formatter, "dyn {}", Listed { separator: Separator::Plus, items: bounds })
            }
            TypeRef::Function(callable) => write!(formatter, "{callable}"),
            TypeRef::Projected(_) | TypeRef::Opaque => formatter.write_str("_"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Returned<'ty>(Option<&'ty TypeRef>);

impl fmt::Display for Returned<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(returned) => write!(formatter, " -> {}", Brief(returned)),
            None => Ok(()),
        }
    }
}

impl fmt::Display for Callable<Sited> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let returned = Returned(Some(self.output.as_ref()).filter(|output| output.unit().is_none()));
        write!(formatter, "fn({}){returned}", Listed { separator: Separator::Comma, items: &self.inputs })
    }
}

impl TypeRef {
    pub(crate) fn innermost(&self) -> Brief<'_> {
        match self {
            TypeRef::Reference(inner) => inner.innermost(),
            TypeRef::Named(_)
            | TypeRef::Tuple(_)
            | TypeRef::Pointer(_)
            | TypeRef::Slice(_)
            | TypeRef::Array(_)
            | TypeRef::Bounded(_)
            | TypeRef::Projected(_)
            | TypeRef::Function(_)
            | TypeRef::Opaque => Brief(self),
        }
    }
}
