mod platform;

use std::path::Path;

use lint_core::RepoFile;
use strum::Display;

use crate::language::{Fix, Language};
use platform::Platform;

#[derive(Display, Clone, Copy, Debug)]
pub(crate) enum Place {
    #[strum(serialize = "source outside assets")]
    Outside,
    #[strum(serialize = "source below assets is not a web asset")]
    Assets,
    #[strum(serialize = "vendored source")]
    Vendored,
    #[strum(serialize = "platform test source")]
    PlatformTests,
    #[strum(serialize = "source below platform is not test code")]
    Platform,
}

impl Place {
    const ASSETS: &'static str = "assets";

    const VENDOR: &'static str = "vendor";

    pub(crate) fn locating(file: &RepoFile) -> Place {
        let under = |directory: &str| {
            file.beneath_named(Path::new(directory)).filter(|below| below.components().next().is_some())
        };
        under(Self::VENDOR)
            .map(|_| Place::Vendored)
            .or_else(|| Platform::tests_covering(file).map(|_| Place::PlatformTests))
            .or_else(|| under(Self::ASSETS).map(|_| Place::Assets))
            .or_else(|| under(Platform::NAME).map(|_| Place::Platform))
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
