use std::iter;

use code_model::{Model, Reach};
use lint_core::SourceFile;

use crate::lowering::Lowered;
use crate::planting::{Placing, Planting};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Index(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Order {
    file: Index,
    module: Index,
}

#[derive(Clone, Copy, Debug)]
struct ListedFile<'read> {
    source: &'read SourceFile,
    reach: Reach,
    order: Order,
}

impl ListedFile<'_> {
    fn lowered(&self) -> FileFacts {
        FileFacts {
            order: self.order,
            placing: Placing::from(self.source),
            lowered: Lowered::of_file(self.source, self.reach),
        }
    }
}

#[derive(Debug)]
pub struct ListedGroup<'read>(Vec<ListedFile<'read>>);

impl ListedGroup<'_> {
    #[must_use]
    pub fn lowered(&self) -> Vec<FileFacts> {
        self.0.iter().map(ListedFile::lowered).collect()
    }
}

#[derive(Debug)]
pub struct Listing<'read>(Vec<ListedGroup<'read>>);

impl<'read> Listing<'read> {
    #[must_use]
    pub fn of_files(files: &'read [SourceFile]) -> Listing<'read> {
        Listing(
            files
                .iter()
                .enumerate()
                .map(|(file, source)| {
                    let reach = source.shipped().map_or(Reach::Test, |_| Reach::Shipped);
                    let members = iter::once((source, reach))
                        .chain(source.test_modules().iter().map(|module| (module, Reach::Test)));
                    ListedGroup(
                        members
                            .enumerate()
                            .map(|(module, (listed, reached))| ListedFile {
                                source: listed,
                                reach: reached,
                                order: Order { file: Index(file), module: Index(module) },
                            })
                            .collect(),
                    )
                })
                .collect(),
        )
    }

    #[must_use]
    pub fn groups(&self) -> &[ListedGroup<'read>] {
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
