mod platform;

use std::fmt;
use std::path::Path;

use lint_core::RepoFile;

use crate::language::{Fix, Language};
use platform::Platform;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Place {
    Outside,
    Assets,
    Vendored,
    PlatformTests,
    Platform,
}

impl Place {
    const ASSETS: &'static str = "assets";

    const VENDOR: &'static str = "vendor";

    pub(crate) fn locating(file: &RepoFile) -> Place {
        file.lies_under(Path::new(Self::VENDOR))
            .map(|_| Place::Vendored)
            .or_else(|| Platform::tests_covering(file).map(|_| Place::PlatformTests))
            .or_else(|| file.lies_under(Path::new(Self::ASSETS)).map(|_| Place::Assets))
            .or_else(|| file.lies_under(Path::new(Platform::NAME)).map(|_| Place::Platform))
            .unwrap_or(Place::Outside)
    }

    pub(crate) fn refusing(self, language: Language) -> Option<Language> {
        match self {
            Place::Outside | Place::Platform => Some(language),
            Place::Vendored | Place::PlatformTests => None,
            Place::Assets => (language.fix() != Fix::MoveToAssets).then_some(language),
        }
    }
}

impl fmt::Display for Place {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Place::Outside => "source outside assets",
            Place::Vendored => "vendored source",
            Place::Assets => "source below assets is not a web asset",
            Place::PlatformTests => "platform test source",
            Place::Platform => "source below platform is not test code",
        })
    }
}
