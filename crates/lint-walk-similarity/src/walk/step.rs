use lint_layout::Name;
use type_facts::{Arity, Implementation, TypeFact};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Step {
    Method { name: Name, arity: Arity, returns: Option<Name> },
    Implements(Name),
}

impl Step {
    const SELF: &'static str = "Self";

    pub(crate) fn walked_by(found: &TypeFact) -> impl Iterator<Item = Step> {
        let own: Name = AsRef::<Name>::as_ref(found).clone();
        let methods = found.methods().iter().map(move |method| Step::Method {
            name: AsRef::<Name>::as_ref(method).clone(),
            arity: method.arity(),
            returns: method.returns().map(|returned| Self::narrowed(returned, &own)),
        });
        let implemented = found.implementations().iter().map(Step::implemented);
        methods.chain(implemented)
    }

    fn implemented(each: &Implementation) -> Step {
        Step::Implements(AsRef::<Name>::as_ref(each).clone())
    }

    fn narrowed(returned: &Name, own: &Name) -> Name {
        if returned == own { Name::from(Self::SELF) } else { returned.clone() }
    }
}
