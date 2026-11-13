use code_model::{Model, Reach};
use lint_core::SourceFile;

use crate::lowering::Lowered;
use crate::planting::{Placing, Planting};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Order(usize);

#[derive(Clone, Copy, Debug)]
pub struct ListedFile<'read> {
    source: &'read SourceFile,
    reach: Reach,
    order: Order,
}

impl ListedFile<'_> {
    #[must_use]
    pub fn lowered(&self) -> FileFacts {
        FileFacts {
            order: self.order,
            placing: Placing::from(self.source),
            lowered: Lowered::of_file(self.source, self.reach),
        }
    }
}

#[derive(Debug)]
pub struct Listing<'read>(Vec<ListedFile<'read>>);

impl<'read> Listing<'read> {
    #[must_use]
    pub fn of_files(files: &'read [SourceFile]) -> Listing<'read> {
        let reached = files.iter().flat_map(|file| {
            let reach = file.shipped().map_or(Reach::Test, |_| Reach::Shipped);
            std::iter::once((file, reach)).chain(file.test_modules().iter().map(|module| (module, Reach::Test)))
        });
        Listing(reached.enumerate().map(|(index, (source, reach))| ListedFile { source, reach, order: Order(index) }).collect())
    }

    #[must_use]
    pub fn entries(&self) -> &[ListedFile<'read>] {
        &self.0
    }
}

#[derive(Debug)]
pub struct FileFacts {
    order: Order,
    placing: Placing,
    lowered: Lowered,
}

impl FileFacts {
    #[must_use]
    pub fn merged(facts: Vec<FileFacts>) -> Model {
        let mut sorted = facts;
        sorted.sort_by_key(|found| found.order);
        let (placings, bodies): (Vec<Placing>, Vec<Lowered>) =
            sorted.into_iter().map(|found| (found.placing, found.lowered)).unzip();
        Planting::merged(&placings, bodies)
    }
}
