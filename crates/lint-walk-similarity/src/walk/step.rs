mod returns;

use std::collections::BTreeSet;

use lint_layout::Name;
use type_facts::{Arity, Facts, Implementation, Method, TypeFact};

use returns::Returns;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Step<'facts> {
    Method { name: &'facts Name, arity: Arity, returns: Option<Returns<'facts>> },
    Implements(&'facts Name),
    Provides { implemented: &'facts Name, name: &'facts Name, arity: Arity, returns: Option<Returns<'facts>> },
}

impl<'facts> Step<'facts> {
    pub(crate) fn walked_by<'file>(
        facts: &'facts Facts<'file>,
        found: &'facts TypeFact<'file>,
        local: &BTreeSet<&Name>,
    ) -> BTreeSet<Step<'facts>> {
        let own: &Name = found.as_ref();
        let inherent = facts.methods(found).iter().map(|method| Self::method(method, own));
        let implemented = facts.implementations(found).iter().flat_map(|each| Self::implemented(each, own, local));
        inherent.chain(implemented).collect()
    }

    fn method(method: &'facts Method, own: &Name) -> Step<'facts> {
        Step::Method {
            name: method.as_ref(),
            arity: method.arity(),
            returns: method.returns().map(|returned| Returns::narrowed(returned, own)),
        }
    }

    fn implemented(
        each: &'facts Implementation,
        own: &'facts Name,
        local: &BTreeSet<&Name>,
    ) -> impl Iterator<Item = Step<'facts>> {
        let implemented: &Name = each.as_ref();
        let provided =
            each.methods().iter().filter(|_| local.contains(implemented)).map(move |method| Step::Provides {
                implemented,
                name: method.as_ref(),
                arity: method.arity(),
                returns: method.returns().map(|returned| Returns::narrowed(returned, own)),
            });
        [Step::Implements(implemented)].into_iter().chain(provided)
    }
}
