use std::collections::BTreeSet;
use std::fmt;

use graph::{Domain, Implementation, ItemPath, Model, Origin, Placed, Reach, Region, Resolved, TypeRef};
use lint_core::{Finding, Fix, Site, Violation};
use lint_layout::{Name, Whole, Word, words};

use crate::transitions::Ground;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Algebra {
    Semigroup,
    Monoid,
}

#[derive(Clone, Copy, Debug)]
enum Std {
    Add,
    Extend,
    FromIterator,
}

#[derive(Debug)]
pub(super) struct Disguised<'model> {
    placed: Placed<'model, &'model Implementation>,
    std: Std,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Declaration<'model> {
    algebra: Algebra,
    subject: &'model Resolved,
}

#[derive(Debug)]
struct Declared<'model> {
    semigroup: Option<&'model Resolved>,
    monoid: Option<&'model Resolved>,
    declarations: BTreeSet<Declaration<'model>>,
}

impl<'model> Disguised<'model> {
    const STD: [(Std, [Word<'static>; 3]); 3] = [
        (Std::Add, words!["std", "ops", "Add"]),
        (Std::Extend, words!["std", "iter", "Extend"]),
        (Std::FromIterator, words!["std", "iter", "FromIterator"]),
    ];
    const OUTPUT: [Word<'static>; 1] = words!["Output"];

    pub(super) fn found(model: &'model Model) -> Vec<Disguised<'model>> {
        let core: Vec<Placed<'model, &'model Implementation>> = model
            .implementations()
            .filter(|placed| placed.entity().origin() == Origin::Written)
            .filter(|placed| AsRef::<Region>::as_ref(placed.entity()).reach() == Reach::Shipped)
            .filter(|placed| Ground::from(*AsRef::<Region>::as_ref(placed.entity())) == Ground::Core)
            .collect();
        let declared = Declared::collected(model, &core);
        core.into_iter()
            .filter_map(|placed| Disguised::std_of(placed).map(|std| Disguised { placed, std }))
            .filter(|disguised| {
                disguised
                    .subject()
                    .is_some_and(|subject| disguised.closing().and_then(Declared::identity) == Some(subject))
            })
            .filter(|disguised| declared.declares(disguised.std.wanted(), disguised.subject()).is_none())
            .collect()
    }

    fn std_of(placed: Placed<'model, &'model Implementation>) -> Option<Std> {
        let implemented = placed.entity().implemented()?;
        let path: &ItemPath = implemented.std_trait().and_then(|_| implemented.std())?;
        Self::STD
            .into_iter()
            .find(|(_, words)| words.iter().copied().collect::<ItemPath>() == *path)
            .map(|(std, _)| std)
    }

    fn subject(&self) -> Option<&'model Resolved> {
        Declared::identity(self.placed.entity().subject_type())
    }

    fn closing(&self) -> Option<&'model TypeRef> {
        match self.std {
            Std::Add => self
                .placed
                .entity()
                .assigned()
                .iter()
                .find(|assigned| AsRef::<Name>::as_ref(*assigned).matches::<Whole>(&Self::OUTPUT).is_some())
                .map(AsRef::<TypeRef>::as_ref),
            Std::Extend | Std::FromIterator => self.placed.entity().interface_arguments().first(),
        }
    }
}

impl<'model> From<&Disguised<'model>> for Site<'model> {
    fn from(disguised: &Disguised<'model>) -> Site<'model> {
        disguised.placed.site()
    }
}

impl<'model> From<&Disguised<'model>> for Violation<'model> {
    fn from(disguised: &Disguised<'model>) -> Violation<'model> {
        Violation::new(
            disguised.placed.site(),
            Hidden { subject: disguised.placed.entity().subject(), std: disguised.std },
        )
    }
}

impl Std {
    fn wanted(self) -> Algebra {
        match self {
            Std::Add => Algebra::Semigroup,
            Std::Extend | Std::FromIterator => Algebra::Monoid,
        }
    }
}

impl<'model> Declared<'model> {
    const SEMIGROUP: [Word<'static>; 2] = words!["algebra", "Semigroup"];
    const MONOID: [Word<'static>; 2] = words!["algebra", "Monoid"];

    fn collected(model: &'model Model, core: &[Placed<'model, &'model Implementation>]) -> Declared<'model> {
        let semigroup = model.exported(Domain::Types, &Self::SEMIGROUP.into_iter().collect());
        let monoid = model.exported(Domain::Types, &Self::MONOID.into_iter().collect());
        let found = Declared { semigroup, monoid, declarations: BTreeSet::new() };
        let declarations = core
            .iter()
            .filter_map(|placed| {
                let implementation = placed.entity();
                let algebra = found.algebra(implementation.implemented()?)?;
                Some(Declaration { algebra, subject: Declared::identity(implementation.subject_type())? })
            })
            .collect();
        Declared { declarations, ..found }
    }

    fn identity(ty: &TypeRef) -> Option<&Resolved> {
        ty.named().map(AsRef::<Resolved>::as_ref)
    }

    fn algebra(&self, implemented: &Resolved) -> Option<Algebra> {
        let semigroup = self.semigroup.filter(|found| *found == implemented).map(|_| Algebra::Semigroup);
        semigroup.or_else(|| self.monoid.filter(|found| *found == implemented).map(|_| Algebra::Monoid))
    }

    fn declares(&self, algebra: Algebra, subject: Option<&'model Resolved>) -> Option<Algebra> {
        subject
            .filter(|resolved| self.declarations.contains(&Declaration { algebra, subject: resolved }))
            .map(|_| algebra)
    }
}

#[derive(Debug)]
struct Hidden<'model> {
    subject: &'model Name,
    std: Std,
}

impl fmt::Display for Hidden<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} implements {} without algebra::{}", self.subject, self.std, self.std.wanted())
    }
}

impl Finding for Hidden<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "declare the algebra this impl hides: implement algebra::Semigroup, and algebra::Monoid when there is an identity, for the type",
        )
    }
}

impl fmt::Display for Std {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Std::Add => write!(formatter, "std::ops::Add with Output = Self"),
            Std::Extend => write!(formatter, "Extend<Self>"),
            Std::FromIterator => write!(formatter, "FromIterator<Self>"),
        }
    }
}

impl fmt::Display for Algebra {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Algebra::Semigroup => write!(formatter, "Semigroup"),
            Algebra::Monoid => write!(formatter, "Monoid"),
        }
    }
}
