use std::path::Path;

use lint_layout::Name;
use lint_layout::Prefix;
use lint_layout::Word;

use super::Directory;

#[derive(Clone, Copy, Debug)]
pub enum Ignored {
    Hidden,
    VersionControl,
    BuildOutput,
}

impl Ignored {
    pub const FOR_PACKAGES: [Ignored; 2] = [Ignored::Hidden, Ignored::BuildOutput];
    pub const FOR_FILES: [Ignored; 2] = [Ignored::VersionControl, Ignored::BuildOutput];

    const BUILD_DIRECTORY: &'static str = "target";
    const MANIFEST: &'static str = "Cargo.toml";
    const HIDDEN: Word<'static> = Word::new(".");
    const VERSION_CONTROL: &'static str = ".git";

    pub fn applying_to(self, directory: &Directory) -> Option<Ignored> {
        let path: &Path = directory.as_ref();
        let named = path.file_name();
        let applies = match self {
            Ignored::Hidden => named.is_some_and(|name| Name::from(name).matches::<Prefix>(&[Self::HIDDEN])),
            Ignored::VersionControl => named.is_some_and(|name| name == Self::VERSION_CONTROL),
            Ignored::BuildOutput => {
                named.is_some_and(|name| name == Self::BUILD_DIRECTORY)
                    && path.parent().is_some_and(|parent| parent.join(Self::MANIFEST).is_file())
            }
        };
        applies.then_some(self)
    }
}
