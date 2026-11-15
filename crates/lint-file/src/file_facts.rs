use lint_core::{Parsed, SourceFile, TypeHomes};
use lint_sweep::{Gathers, Sweep, Watching};

#[derive(Debug)]
pub struct FileFacts<'head> {
    homes: TypeHomes<'head>,
    shipped: Option<()>,
}

impl<'head> From<Parsed<'_, 'head>> for FileFacts<'head> {
    fn from(parsed: Parsed<'_, 'head>) -> FileFacts<'head> {
        FileFacts { homes: TypeHomes::from(parsed), shipped: parsed.file().shipped().map(|_| ()) }
    }
}

impl<'head> AsRef<TypeHomes<'head>> for FileFacts<'head> {
    fn as_ref(&self) -> &TypeHomes<'head> {
        &self.homes
    }
}

impl<'head> FileFacts<'head> {
    pub fn watching(&mut self) -> Vec<&mut Watching<'_>> {
        let homes: Vec<&mut Watching<'_>> = vec![&mut self.homes];
        self.shipped.map_or_default(|()| homes)
    }

    pub fn completed<Part>(
        file: &SourceFile,
        parsed: Parsed<'_, 'head>,
        mut gathering: impl Gathers<Part>,
        complete: impl FnOnce(Part, &FileFacts<'head>) -> Part,
    ) -> Part {
        let mut facts = FileFacts::from(parsed);
        let watching = gathering.watching().into_iter().chain(facts.watching()).collect();
        Sweep::watching(file, watching);
        complete(gathering.gathered(), &facts)
    }
}
