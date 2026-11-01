use std::collections::BTreeSet;

use lint_layout::Name;
use type_facts::{Arity, Implementation, Method, TypeFact};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Step {
    Method { name: Name, arity: Arity, returns: Option<Name> },
    Implements(Name),
    Provides { implemented: Name, name: Name, arity: Arity, returns: Option<Name> },
}

impl Step {
    const SELF: &'static str = "Self";

    pub(crate) fn walked_by(found: &TypeFact, local: &BTreeSet<Name>) -> Vec<Step> {
        let own: &Name = found.as_ref();
        let inherent = found.methods().iter().map(|method| Self::method(method, own));
        let implemented = found.implementations().iter().flat_map(|each| Self::implemented(each, own, local));
        inherent.chain(implemented).collect()
    }

    fn method(method: &Method, own: &Name) -> Step {
        Step::Method {
            name: AsRef::<Name>::as_ref(method).clone(),
            arity: method.arity(),
            returns: method.returns().map(|returned| Self::narrowed(returned, own)),
        }
    }

    fn implemented(each: &Implementation, own: &Name, local: &BTreeSet<Name>) -> Vec<Step> {
        let implemented: &Name = each.as_ref();
        let provided = each.methods().iter().filter(|_| local.contains(implemented)).map(|method| Step::Provides {
            implemented: implemented.clone(),
            name: AsRef::<Name>::as_ref(method).clone(),
            arity: method.arity(),
            returns: method.returns().map(|returned| Self::narrowed(returned, own)),
        });
        [Step::Implements(implemented.clone())].into_iter().chain(provided).collect()
    }

    fn narrowed(returned: &Name, own: &Name) -> Name {
        if returned == own { Name::from(Self::SELF) } else { returned.clone() }
    }
}
