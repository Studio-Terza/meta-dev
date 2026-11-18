mod emitters;
mod path_reach;
mod test_code;

use emitters::{EmitterScan, Emitters};
use lint_core::Fix;
use lint_core::Message;
use lint_core::Problem;
use lint_core::{Headline, Parsed};
use lint_core::{SourceFile, Violation};
use lint_file::{ContextLint, Reads, WatchingLint};
use lint_sweep::{Contributing, Gathers, Watching};
use path_reach::PathReach;
use test_code::{TestCode, Tested};

#[derive(Debug)]
pub struct PublicApiTests;

impl PublicApiTests {
    pub(crate) const IN_SOURCE: Message = Message::new(
        Problem::new("test code in src"),
        Fix::new("move it to the package's tests directory and call the public API from there"),
    );
    pub(crate) const REACH: Message = Message::new(
        Problem::new("test file includes a module from src with #[path]"),
        Fix::new("call the public API instead"),
    );
}

impl ContextLint for PublicApiTests {
    const HEADLINE: Headline = Headline::new("tests go through public APIs");
    const READS: Reads = Reads::Every;

    type Part<'head> = Emitters;
    type Context<'head> = Emitters;

    type Gathering<'head> = Contributing<EmitterScan>;

    fn gathering(_: Parsed<'_, '_>) -> Contributing<EmitterScan> {
        Contributing::new(EmitterScan)
    }
}

impl WatchingLint for PublicApiTests {
    type Found<'file> = Tested<'file>;
    type Watched<'file> = Watched<'file>;

    fn watched(file: &SourceFile) -> Watched<'_> {
        file.shipped().map_or_else(
            || Watched::Beside(Contributing::new(PathReach::beside(file))),
            |_| Watched::Shipped(Contributing::new(TestCode::from(file))),
        )
    }

    fn judged<'file, 'head: 'file>(
        _: &'file SourceFile,
        found: Tested<'file>,
        emitters: &Emitters,
    ) -> Vec<Violation<'file>> {
        found.emitted(emitters.as_ref())
    }
}

#[derive(Debug)]
pub enum Watched<'file> {
    Shipped(Contributing<TestCode<'file>>),
    Beside(Contributing<PathReach<'file>>),
}

impl<'file> Gathers<Tested<'file>> for Watched<'file> {
    fn watching(&mut self) -> impl IntoIterator<Item = &mut Watching<'_>> {
        match self {
            Watched::Shipped(code) => std::iter::once::<&mut Watching<'_>>(code),
            Watched::Beside(reach) => std::iter::once::<&mut Watching<'_>>(reach),
        }
    }

    fn gathered(self) -> Tested<'file> {
        match self {
            Watched::Shipped(contributing) => contributing.concluded(|code, (), in_source| code.tested(in_source)),
            Watched::Beside(contributing) => contributing.concluded(|reach, (), reached| reach.tested(reached)),
        }
    }
}
