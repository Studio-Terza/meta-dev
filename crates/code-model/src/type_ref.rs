use lint_layout::Name;

use crate::model::Aliases;
use crate::resolved::Resolved;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Steps(u8);

impl Steps {
    pub(crate) const LIMIT: Steps = Steps(16);

    fn taken(self) -> Option<Steps> {
        self.0.checked_sub(1).map(Steps)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeRef<Bound = Resolved> {
    Named(Named<Bound>),
    Tuple(Vec<TypeRef<Bound>>),
    Opaque,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Named<Bound> {
    path: Bound,
    arguments: Vec<TypeRef<Bound>>,
}

impl<Bound> Named<Bound> {
    #[must_use]
    pub fn new(path: Bound, arguments: Vec<TypeRef<Bound>>) -> Named<Bound> {
        Named { path, arguments }
    }

    #[must_use]
    pub fn arguments(&self) -> &[TypeRef<Bound>] {
        &self.arguments
    }

    fn mapped<Into>(self, resolve: &mut impl FnMut(Bound) -> Into) -> Named<Into> {
        Named {
            path: resolve(self.path),
            arguments: self.arguments.into_iter().map(|argument| argument.mapped(resolve)).collect(),
        }
    }
}

impl<Bound> AsRef<Bound> for Named<Bound> {
    fn as_ref(&self) -> &Bound {
        &self.path
    }
}

impl Named<Resolved> {
    fn substituted(&self, parameters: &[Name], arguments: &[TypeRef]) -> TypeRef {
        let bound = self
            .path
            .parameter()
            .and_then(|name| parameters.iter().position(|parameter| parameter == name))
            .and_then(|at| arguments.get(at));
        bound.cloned().unwrap_or_else(|| {
            TypeRef::Named(Named {
                path: self.path.clone(),
                arguments: self.arguments.iter().map(|argument| argument.substituted(parameters, arguments)).collect(),
            })
        })
    }

    fn expanded(&self, aliases: &Aliases, steps: Steps) -> Option<TypeRef> {
        match aliases.aliasing(&self.path) {
            Some(alias) => {
                alias.target().substituted(alias.parameters(), &self.arguments).expanded(aliases, steps.taken()?)
            }
            None => TypeRef::each_expanded(&self.arguments, aliases, steps)
                .map(|arguments| TypeRef::Named(Named { path: self.path.clone(), arguments })),
        }
    }
}

impl<Bound> TypeRef<Bound> {
    pub fn mapped<Into>(self, resolve: &mut impl FnMut(Bound) -> Into) -> TypeRef<Into> {
        match self {
            TypeRef::Named(named) => TypeRef::Named(named.mapped(resolve)),
            TypeRef::Tuple(elements) => {
                TypeRef::Tuple(elements.into_iter().map(|element| element.mapped(resolve)).collect())
            }
            TypeRef::Opaque => TypeRef::Opaque,
        }
    }

    #[must_use]
    pub fn named(&self) -> Option<&Named<Bound>> {
        match self {
            TypeRef::Named(named) => Some(named),
            TypeRef::Tuple(_) | TypeRef::Opaque => None,
        }
    }

    #[must_use]
    pub fn unit(&self) -> Option<&TypeRef<Bound>> {
        match self {
            TypeRef::Tuple(elements) => elements.first().xor(Some(self)),
            TypeRef::Named(_) | TypeRef::Opaque => None,
        }
    }
}

impl TypeRef {
    #[must_use]
    pub fn substituted(&self, parameters: &[Name], arguments: &[TypeRef]) -> TypeRef {
        match self {
            TypeRef::Named(named) => named.substituted(parameters, arguments),
            TypeRef::Tuple(elements) => {
                TypeRef::Tuple(elements.iter().map(|element| element.substituted(parameters, arguments)).collect())
            }
            TypeRef::Opaque => TypeRef::Opaque,
        }
    }

    pub(crate) fn expanded(&self, aliases: &Aliases, steps: Steps) -> Option<TypeRef> {
        match self {
            TypeRef::Named(named) => named.expanded(aliases, steps),
            TypeRef::Tuple(elements) => TypeRef::each_expanded(elements, aliases, steps).map(TypeRef::Tuple),
            TypeRef::Opaque => Some(TypeRef::Opaque),
        }
    }

    fn each_expanded(types: &[TypeRef], aliases: &Aliases, steps: Steps) -> Option<Vec<TypeRef>> {
        types.iter().map(|ty| ty.expanded(aliases, steps)).collect()
    }
}
