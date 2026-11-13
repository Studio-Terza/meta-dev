use std::iter;
use std::path::PathBuf;

use graph::{Model, Reach};
use lint_core::SourceFile;

use crate::lowering::Lowered;
use crate::planting::{Placing, Planting};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Index(usize);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Order {
    file: PathBuf,
    module: Index,
}

#[derive(Clone, Debug)]
struct ListedFile<'read> {
    source: &'read SourceFile,
    reach: Reach,
    order: Order,
}

impl ListedFile<'_> {
    fn lowered(&self) -> FileFacts {
        FileFacts {
            order: self.order.clone(),
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
                .map(|source| {
                    let reach = source.shipped().map_or(Reach::Test, |_| Reach::Shipped);
                    let members = iter::once((source, reach))
                        .chain(source.test_modules().iter().map(|module| (module, Reach::Test)));
                    ListedGroup(
                        members
                            .enumerate()
                            .map(|(module, (listed, reached))| ListedFile {
                                source: listed,
                                reach: reached,
                                order: Order { file: source.head().path().to_path_buf(), module: Index(module) },
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
        sorted.sort_by(|earlier, later| earlier.order.cmp(&later.order));
        let (placings, bodies): (Vec<Placing>, Vec<Lowered>) =
            sorted.into_iter().map(|found| (found.placing, found.lowered)).unzip();
        Planting::merged(&placings, bodies)
    }
}
