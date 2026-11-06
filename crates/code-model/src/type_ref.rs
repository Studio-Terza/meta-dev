use lint_layout::Name;

use crate::item::AliasParameter;
use crate::model::Aliases;
use crate::resolved::Resolved;
use crate::sited::Sited;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Steps(u8);

impl Steps {
    pub(crate) const LIMIT: Steps = Steps(16);

    fn taken(self) -> Option<Steps> {
        self.0.checked_sub(1).map(Steps)
    }
}

pub trait TypeUses {
    fn named(&mut self, named: &Named<Sited>);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeRef<Bound = Sited> {
    Named(Named<Bound>),
    Tuple(Vec<TypeRef<Bound>>),
    Reference(Box<TypeRef<Bound>>),
    Pointer(Box<TypeRef<Bound>>),
    Slice(Box<TypeRef<Bound>>),
    Array(Arrayed<Bound>),
    Bounded(Bounds<Bound>),
    Projected(Box<Projection<Bound>>),
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
pub struct Bounds<Bound> {
    traits: Vec<Named<Bound>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Projection<Bound> {
    subject: TypeRef<Bound>,
    interface: Option<Named<Bound>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Callable<Bound> {
    inputs: Vec<TypeRef<Bound>>,
    output: Box<TypeRef<Bound>>,
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

    fn mapped<Into>(self, resolve: &mut impl FnMut(Bound) -> Into) -> Named<Into> {
        Named {
            path: resolve(self.path),
            arguments: TypeRef::each_mapped(self.arguments, resolve),
            bindings: TypeRef::each_mapped(self.bindings, resolve),
        }
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
        self.arguments.iter().chain(&self.bindings).for_each(|ty| ty.each_use(uses));
    }

    pub(crate) fn substituted(&self, parameters: &[AliasParameter], arguments: &[TypeRef]) -> TypeRef {
        let resolved: &Resolved = self.path.as_ref();
        let declared = resolved.parameter().and_then(|name| {
            parameters.iter().enumerate().find(|(_, parameter)| AsRef::<Name>::as_ref(*parameter) == name)
        });
        let bound = declared.and_then(|(at, _)| arguments.get(at));
        bound.cloned().unwrap_or_else(|| TypeRef::Named(self.each_substituted(parameters, arguments)))
    }

    fn each_substituted(&self, parameters: &[AliasParameter], arguments: &[TypeRef]) -> Named<Sited> {
        Named {
            path: self.path.clone(),
            arguments: TypeRef::each_substituted(&self.arguments, parameters, arguments),
            bindings: TypeRef::each_substituted(&self.bindings, parameters, arguments),
        }
    }

    fn expansion(&self, aliases: &Aliases) -> Option<TypeRef> {
        aliases.aliasing(self.path.as_ref()).map(|alias| alias.expansion(&self.arguments))
    }

    pub(crate) fn expanded(&self, aliases: &Aliases, steps: Steps) -> Option<TypeRef> {
        match self.expansion(aliases) {
            Some(expansion) => expansion.expanded(aliases, steps.taken()?),
            None => self.each_expanded(aliases, steps).map(TypeRef::Named),
        }
    }

    fn each_expanded(&self, aliases: &Aliases, steps: Steps) -> Option<Named<Sited>> {
        Some(Named {
            path: self.path.clone(),
            arguments: TypeRef::each_expanded(&self.arguments, aliases, steps)?,
            bindings: TypeRef::each_expanded(&self.bindings, aliases, steps)?,
        })
    }
}

impl<Bound> Arrayed<Bound> {
    #[must_use]
    pub fn new(element: TypeRef<Bound>, measured: Vec<TypeRef<Bound>>) -> Arrayed<Bound> {
        Arrayed { element: Box::new(element), measured }
    }

    fn mapped<Into>(self, resolve: &mut impl FnMut(Bound) -> Into) -> Arrayed<Into> {
        Arrayed {
            element: Box::new(self.element.mapped(resolve)),
            measured: TypeRef::each_mapped(self.measured, resolve),
        }
    }
}

impl Arrayed<Sited> {
    fn each_use(&self, uses: &mut impl TypeUses) {
        [&*self.element].into_iter().chain(&self.measured).fold(uses, |uses, ty| {
            ty.each_use(uses);
            uses
        });
    }

    fn substituted(&self, parameters: &[AliasParameter], arguments: &[TypeRef]) -> Arrayed<Sited> {
        Arrayed {
            element: Box::new(self.element.substituted(parameters, arguments)),
            measured: TypeRef::each_substituted(&self.measured, parameters, arguments),
        }
    }

    fn expanded(&self, aliases: &Aliases, steps: Steps) -> Option<Arrayed<Sited>> {
        Some(Arrayed {
            element: Box::new(self.element.expanded(aliases, steps)?),
            measured: TypeRef::each_expanded(&self.measured, aliases, steps)?,
        })
    }
}

impl<Bound> From<Vec<Named<Bound>>> for Bounds<Bound> {
    fn from(traits: Vec<Named<Bound>>) -> Bounds<Bound> {
        Bounds { traits }
    }
}

impl<Bound> Bounds<Bound> {
    fn mapped<Into>(self, resolve: &mut impl FnMut(Bound) -> Into) -> Bounds<Into> {
        Bounds { traits: self.traits.into_iter().map(|bound| bound.mapped(resolve)).collect() }
    }
}

impl Bounds<Sited> {
    fn each_use(&self, uses: &mut impl TypeUses) {
        self.traits.iter().fold(uses, |uses, bound| {
            uses.named(bound);
            uses
        });
    }

    fn substituted(&self, parameters: &[AliasParameter], arguments: &[TypeRef]) -> Bounds<Sited> {
        Bounds { traits: self.traits.iter().map(|bound| bound.each_substituted(parameters, arguments)).collect() }
    }

    fn expanded(&self, aliases: &Aliases, steps: Steps) -> Option<Bounds<Sited>> {
        let traits: Option<Vec<Named<Sited>>> =
            self.traits.iter().map(|bound| bound.each_expanded(aliases, steps)).collect();
        traits.map(|traits| Bounds { traits })
    }
}

impl<Bound> Projection<Bound> {
    #[must_use]
    pub fn new(subject: TypeRef<Bound>, interface: Option<Named<Bound>>) -> Projection<Bound> {
        Projection { subject, interface }
    }

    fn mapped<Into>(self, resolve: &mut impl FnMut(Bound) -> Into) -> Projection<Into> {
        Projection {
            subject: self.subject.mapped(resolve),
            interface: self.interface.map(|interface| interface.mapped(resolve)),
        }
    }
}

impl Projection<Sited> {
    fn each_use(&self, uses: &mut impl TypeUses) {
        self.subject.each_use(uses);
        self.interface.iter().fold(uses, |uses, interface| {
            uses.named(interface);
            uses
        });
    }

    fn substituted(&self, parameters: &[AliasParameter], arguments: &[TypeRef]) -> Projection<Sited> {
        Projection {
            subject: self.subject.substituted(parameters, arguments),
            interface: self.interface.as_ref().map(|interface| interface.each_substituted(parameters, arguments)),
        }
    }

    fn expanded(&self, aliases: &Aliases, steps: Steps) -> Option<Projection<Sited>> {
        let interface: Option<Named<Sited>> = self
            .interface
            .iter()
            .map(|interface| interface.each_expanded(aliases, steps))
            .collect::<Option<Vec<Named<Sited>>>>()?
            .into_iter()
            .next();
        Some(Projection { subject: self.subject.expanded(aliases, steps)?, interface })
    }
}

impl<Bound> Callable<Bound> {
    #[must_use]
    pub fn new(inputs: Vec<TypeRef<Bound>>, output: TypeRef<Bound>) -> Callable<Bound> {
        Callable { inputs, output: Box::new(output) }
    }

    fn mapped<Into>(self, resolve: &mut impl FnMut(Bound) -> Into) -> Callable<Into> {
        Callable { inputs: TypeRef::each_mapped(self.inputs, resolve), output: Box::new(self.output.mapped(resolve)) }
    }
}

impl Callable<Sited> {
    fn each_use(&self, uses: &mut impl TypeUses) {
        self.inputs.iter().chain([&*self.output]).for_each(|ty| ty.each_use(uses));
    }

    fn substituted(&self, parameters: &[AliasParameter], arguments: &[TypeRef]) -> Callable<Sited> {
        Callable {
            inputs: TypeRef::each_substituted(&self.inputs, parameters, arguments),
            output: Box::new(self.output.substituted(parameters, arguments)),
        }
    }

    fn expanded(&self, aliases: &Aliases, steps: Steps) -> Option<Callable<Sited>> {
        Some(Callable {
            inputs: TypeRef::each_expanded(&self.inputs, aliases, steps)?,
            output: Box::new(self.output.expanded(aliases, steps)?),
        })
    }
}

impl<Bound> TypeRef<Bound> {
    pub fn mapped<Into>(self, resolve: &mut impl FnMut(Bound) -> Into) -> TypeRef<Into> {
        match self {
            TypeRef::Named(named) => TypeRef::Named(named.mapped(resolve)),
            TypeRef::Tuple(elements) => TypeRef::Tuple(TypeRef::each_mapped(elements, resolve)),
            TypeRef::Reference(pointee) => TypeRef::Reference(Box::new(pointee.mapped(resolve))),
            TypeRef::Pointer(pointee) => TypeRef::Pointer(Box::new(pointee.mapped(resolve))),
            TypeRef::Slice(element) => TypeRef::Slice(Box::new(element.mapped(resolve))),
            TypeRef::Array(arrayed) => TypeRef::Array(arrayed.mapped(resolve)),
            TypeRef::Bounded(bounds) => TypeRef::Bounded(bounds.mapped(resolve)),
            TypeRef::Projected(projection) => TypeRef::Projected(Box::new(projection.mapped(resolve))),
            TypeRef::Function(callable) => TypeRef::Function(callable.mapped(resolve)),
            TypeRef::Opaque => TypeRef::Opaque,
        }
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

    fn each_mapped<Into>(types: Vec<TypeRef<Bound>>, resolve: &mut impl FnMut(Bound) -> Into) -> Vec<TypeRef<Into>> {
        types.into_iter().map(|ty| ty.mapped(resolve)).collect()
    }
}

impl TypeRef {
    pub fn each_use(&self, uses: &mut impl TypeUses) {
        match self {
            TypeRef::Named(named) => uses.named(named),
            TypeRef::Tuple(elements) => TypeRef::each_used(elements, uses),
            TypeRef::Reference(element) | TypeRef::Pointer(element) | TypeRef::Slice(element) => element.each_use(uses),
            TypeRef::Array(arrayed) => arrayed.each_use(uses),
            TypeRef::Bounded(bounds) => bounds.each_use(uses),
            TypeRef::Projected(projection) => projection.each_use(uses),
            TypeRef::Function(callable) => callable.each_use(uses),
            TypeRef::Opaque => (),
        }
    }

    #[must_use]
    pub fn substituted(&self, parameters: &[AliasParameter], arguments: &[TypeRef]) -> TypeRef {
        match self {
            TypeRef::Named(named) => named.substituted(parameters, arguments),
            TypeRef::Tuple(elements) => TypeRef::Tuple(TypeRef::each_substituted(elements, parameters, arguments)),
            TypeRef::Reference(pointee) => TypeRef::Reference(Box::new(pointee.substituted(parameters, arguments))),
            TypeRef::Pointer(pointee) => TypeRef::Pointer(Box::new(pointee.substituted(parameters, arguments))),
            TypeRef::Slice(element) => TypeRef::Slice(Box::new(element.substituted(parameters, arguments))),
            TypeRef::Array(arrayed) => TypeRef::Array(arrayed.substituted(parameters, arguments)),
            TypeRef::Bounded(bounds) => TypeRef::Bounded(bounds.substituted(parameters, arguments)),
            TypeRef::Projected(projection) => {
                TypeRef::Projected(Box::new(projection.substituted(parameters, arguments)))
            }
            TypeRef::Function(callable) => TypeRef::Function(callable.substituted(parameters, arguments)),
            TypeRef::Opaque => TypeRef::Opaque,
        }
    }

    pub(crate) fn expanded(&self, aliases: &Aliases, steps: Steps) -> Option<TypeRef> {
        match self {
            TypeRef::Named(named) => named.expanded(aliases, steps),
            TypeRef::Tuple(elements) => TypeRef::each_expanded(elements, aliases, steps).map(TypeRef::Tuple),
            TypeRef::Reference(pointee) => pointee.expanded(aliases, steps).map(Box::new).map(TypeRef::Reference),
            TypeRef::Pointer(pointee) => pointee.expanded(aliases, steps).map(Box::new).map(TypeRef::Pointer),
            TypeRef::Slice(element) => element.expanded(aliases, steps).map(Box::new).map(TypeRef::Slice),
            TypeRef::Array(arrayed) => arrayed.expanded(aliases, steps).map(TypeRef::Array),
            TypeRef::Bounded(bounds) => bounds.expanded(aliases, steps).map(TypeRef::Bounded),
            TypeRef::Projected(projection) => projection.expanded(aliases, steps).map(Box::new).map(TypeRef::Projected),
            TypeRef::Function(callable) => callable.expanded(aliases, steps).map(TypeRef::Function),
            TypeRef::Opaque => Some(TypeRef::Opaque),
        }
    }

    fn each_used(types: &[TypeRef], uses: &mut impl TypeUses) {
        types.iter().fold(uses, |uses, ty| {
            ty.each_use(uses);
            uses
        });
    }

    fn each_substituted(types: &[TypeRef], parameters: &[AliasParameter], arguments: &[TypeRef]) -> Vec<TypeRef> {
        types.iter().map(|ty| ty.substituted(parameters, arguments)).collect()
    }

    fn each_expanded(types: &[TypeRef], aliases: &Aliases, steps: Steps) -> Option<Vec<TypeRef>> {
        types.iter().map(|ty| ty.expanded(aliases, steps)).collect()
    }
}
