use std::collections::BTreeSet;

use graph::{External, Implementation, ItemId, ItemPath, Model, Placed, Resolved, TypeRef};
use lint_layout::{Word, words};

#[derive(Debug)]
pub(crate) struct ErrorTypes {
    crate_items: BTreeSet<ItemId>,
    std_errors: BTreeSet<ItemPath>,
}

impl From<&Model> for ErrorTypes {
    fn from(model: &Model) -> ErrorTypes {
        let traits: BTreeSet<ItemPath> =
            BTreeSet::from([Self::STD_ERROR.into_iter().collect(), Self::THISERROR.into_iter().collect()]);
        let implemented = model
            .implementations()
            .map(Placed::entity)
            .filter_map(|implementation| Self::implementing(&traits, implementation))
            .map(|subject| model.expanded(subject));
        let result: ItemPath = Self::RESULT.into_iter().collect();
        let returned = model
            .functions()
            .filter_map(|found| found.entity().signed_here())
            .filter_map(|signature| signature.output().written())
            .map(|output| model.expanded(output))
            .filter_map(|output| Self::failure(&result, &output));
        let crate_items = implemented
            .chain(returned)
            .filter_map(|ty| ty.named().and_then(|named| Resolved::crate_item(named.as_ref())))
            .collect();
        let std_errors = Self::STD_ERRORS.iter().map(|error| error.iter().copied().collect()).collect();
        ErrorTypes { crate_items, std_errors }
    }
}

impl ErrorTypes {
    const STD_ERROR: [Word<'static>; 3] = words!["std", "error", "Error"];
    const THISERROR: [Word<'static>; 2] = words!["thiserror", "Error"];
    const RESULT: [Word<'static>; 3] = words!["std", "result", "Result"];
    const STD_ERRORS: [[Word<'static>; 3]; 2] = [words!["std", "io", "Error"], words!["std", "fmt", "Error"]];

    pub(crate) fn holding<'ty>(&self, ty: &'ty TypeRef) -> Option<&'ty TypeRef> {
        let path: &Resolved = ty.named()?.as_ref();
        let declared = path.crate_item().and_then(|item| self.crate_items.get(&item)).map(|_| ty);
        declared.or_else(|| {
            let std = path.std()?;
            self.std_errors.get(std).map(|_| ty)
        })
    }

    fn implementing<'model>(
        traits: &BTreeSet<ItemPath>,
        implementation: &'model Implementation,
    ) -> Option<&'model TypeRef> {
        let implemented = implementation.implemented()?;
        let path = implemented.std().or_else(|| implemented.external().map(External::path))?;
        traits.get(path).map(|_| implementation.subject_type())
    }

    fn failure(result: &ItemPath, output: &TypeRef) -> Option<TypeRef> {
        let named = output.named()?;
        let std = Resolved::std(named.as_ref())?;
        [std].into_iter().find(|path| *path == result).and_then(|_| named.arguments().get(1)).cloned()
    }
}
