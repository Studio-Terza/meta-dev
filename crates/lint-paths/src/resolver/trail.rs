use crate::module::Module;
use lint_layout::Name;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Globbing {
    #[default]
    Open,
    Spent,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Trail<'trail> {
    followed: Option<Followed<'trail>>,
    globbing: Globbing,
}

#[derive(Clone, Copy, Debug)]
struct Followed<'trail> {
    earlier: &'trail Trail<'trail>,
    module: &'trail Module<'trail>,
    name: &'trail Name,
}

impl<'trail> Trail<'trail> {
    pub(crate) fn followed(&self, module: &Module<'_>, name: &Name) -> Option<&'trail Name> {
        let step = self.followed?;
        Some(step.name)
            .filter(|&followed| followed == name)
            .filter(|_| step.module == module)
            .or_else(|| step.earlier.followed(module, name))
    }

    pub(crate) fn following(&'trail self, module: &'trail Module<'trail>, name: &'trail Name) -> Trail<'trail> {
        Trail { followed: Some(Followed { earlier: self, module, name }), globbing: self.globbing }
    }

    pub(crate) fn spent(&self) -> Trail<'trail> {
        Trail { followed: self.followed, globbing: Globbing::Spent }
    }

    pub(crate) fn globbing(&self) -> Globbing {
        self.globbing
    }
}
