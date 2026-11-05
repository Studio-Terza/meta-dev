use syn::{Path, Type};

use lint_layout::Name;

use crate::transitions::collector::origins::Origins;

#[derive(Clone, Debug, Default)]
pub(super) struct Expanding(Vec<Name>);

impl Expanding {
    pub(super) fn through<'found, Read>(
        &self,
        path: &Path,
        origins: &'found Origins<'_>,
        read: Read,
    ) -> Option<&'found Type>
    where
        Read: FnOnce(&'found Type, &Expanding) -> Option<&'found Type>,
    {
        let name = path.segments.first().filter(|_| path.segments.len() == 1).map(|only| Name::from(&only.ident))?;
        let unseen = Some(&name).filter(|candidate| self.0.iter().all(|seen| seen != *candidate))?;
        let target = origins.aliased(unseen)?;
        read(target, &Expanding(self.0.iter().cloned().chain(Some(name)).collect()))
    }
}
