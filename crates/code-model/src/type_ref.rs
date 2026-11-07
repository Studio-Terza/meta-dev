use std::convert::Infallible;

use crate::resolved::Resolved;
use crate::sited::Sited;

pub trait TypeUses {
    fn named(&mut self, named: &Named<Sited>);
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
    Tuple(Vec<Self>),
    Reference(Box<Self>),
    Pointer(Box<Self>),
    Slice(Box<Self>),
    Array(Arrayed<Bound>),
    Bounded(Vec<Self>),
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
    pub const fn new(path: Bound, arguments: Vec<TypeRef<Bound>>) -> Self {
        Self { path, arguments, bindings: Vec::new() }
    }

    #[must_use]
    pub const fn bound(path: Bound, arguments: Vec<TypeRef<Bound>>, bindings: Vec<TypeRef<Bound>>) -> Self {
        Self { path, arguments, bindings }
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
}

impl<Bound> Arrayed<Bound> {
    #[must_use]
    pub fn new(element: TypeRef<Bound>, measured: Vec<TypeRef<Bound>>) -> Self {
        Self { element: Box::new(element), measured }
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
    pub fn new(subject: TypeRef<Bound>, arguments: Vec<TypeRef<Bound>>) -> Self {
        Self { subject: Box::new(subject), arguments }
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
    pub fn new(inputs: Vec<TypeRef<Bound>>, output: TypeRef<Bound>) -> Self {
        Self { inputs, output: Box::new(output) }
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
    pub const fn named(&self) -> Option<&Named<Bound>> {
        match self {
            Self::Named(named) => Some(named),
            Self::Tuple(_)
            | Self::Reference(_)
            | Self::Pointer(_)
            | Self::Slice(_)
            | Self::Array(_)
            | Self::Bounded(_)
            | Self::Projected(_)
            | Self::Function(_)
            | Self::Opaque => None,
        }
    }

    #[must_use]
    pub fn unit(&self) -> Option<&Self> {
        match self {
            Self::Tuple(elements) => elements.first().xor(Some(self)),
            Self::Named(_)
            | Self::Reference(_)
            | Self::Pointer(_)
            | Self::Slice(_)
            | Self::Array(_)
            | Self::Bounded(_)
            | Self::Projected(_)
            | Self::Function(_)
            | Self::Opaque => None,
        }
    }

    #[must_use]
    pub fn respelled(self, respell: impl FnOnce(Bound) -> Bound) -> Self {
        match self {
            Self::Named(named) => {
                Self::Named(Named { path: respell(named.path), arguments: named.arguments, bindings: named.bindings })
            }
            other => other,
        }
    }

    pub(crate) fn rebuilt<Rule: Rebuild<Bound>>(&self, rule: &mut Rule) -> Result<TypeRef<Rule::Into>, Rule::Fail> {
        match self {
            Self::Named(named) => named.rebuilt(rule).map(TypeRef::Named),
            Self::Tuple(elements) => Self::each_rebuilt(elements, rule).map(TypeRef::Tuple),
            Self::Reference(inner) => rule.child(inner).map(Box::new).map(TypeRef::Reference),
            Self::Pointer(inner) => rule.child(inner).map(Box::new).map(TypeRef::Pointer),
            Self::Slice(inner) => rule.child(inner).map(Box::new).map(TypeRef::Slice),
            Self::Array(arrayed) => arrayed.rebuilt(rule).map(TypeRef::Array),
            Self::Bounded(bounds) => Self::each_rebuilt(bounds, rule).map(TypeRef::Bounded),
            Self::Projected(projection) => projection.rebuilt(rule).map(TypeRef::Projected),
            Self::Function(callable) => callable.rebuilt(rule).map(TypeRef::Function),
            Self::Opaque => Ok(TypeRef::Opaque),
        }
    }

    fn parts(&self) -> Parts<'_, Bound> {
        match self {
            Self::Named(named) => named.parts(),
            Self::Array(arrayed) => arrayed.parts(),
            Self::Projected(projection) => projection.parts(),
            Self::Function(callable) => callable.parts(),
            Self::Tuple(elements) | Self::Bounded(elements) => Parts { first: elements, rest: &[] },
            Self::Reference(inner) | Self::Pointer(inner) | Self::Slice(inner) => {
                Parts { first: std::slice::from_ref(&**inner), rest: &[] }
            }
            Self::Opaque => Parts { first: &[], rest: &[] },
        }
    }

    fn each_rebuilt<Rule: Rebuild<Bound>>(
        types: &[Self],
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
