use std::iter;
use std::path::PathBuf;

use graph::{Model, Reach};
use lint_core::SourceFile;

use crate::lowering::{Lowered, Lowering};
use crate::planting::{Placing, Planting};
use lint_sweep::{Gathers, Sweep, Watching};

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

impl<'read> ListedFile<'read> {
    fn lowering(&self) -> FileLowering<'read> {
        FileLowering {
            source: self.source,
            order: self.order.clone(),
            lowering: Lowering::of_file(self.source, self.reach),
        }
    }
}

#[derive(Debug)]
struct FileLowering<'read> {
    source: &'read SourceFile,
    order: Order,
    lowering: Lowering,
}

impl Gathers<FileFacts> for FileLowering<'_> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        Sweep::alone(&mut self.lowering)
    }

    fn gathered(self) -> FileFacts {
        FileFacts { order: self.order, placing: Placing::from(self.source), lowered: self.lowering.lowered() }
    }
}

#[derive(Debug)]
pub struct GroupLowering<'read> {
    head: Option<FileLowering<'read>>,
    modules: Vec<FileLowering<'read>>,
}

impl Gathers<Vec<FileFacts>> for GroupLowering<'_> {
    fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        self.head.as_mut().map_or_default(Gathers::watching)
    }

    fn gathered(self) -> Vec<FileFacts> {
        let swept = self.modules.into_iter().map(|module| Sweep::gathered(module.source, module));
        self.head.into_iter().map(Gathers::gathered).chain(swept).collect()
    }
}

#[derive(Debug)]
pub struct ListedGroup<'read>(Vec<ListedFile<'read>>);

impl<'read> ListedGroup<'read> {
    #[must_use]
    pub fn lowering(&self) -> GroupLowering<'read> {
        GroupLowering {
            head: self.0.first().map(ListedFile::lowering),
            modules: self.0.iter().skip(1).map(ListedFile::lowering).collect(),
        }
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
