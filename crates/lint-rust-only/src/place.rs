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

    pub(crate) fn locating(file: &RepoFile) -> Self {
        file.lies_under(Path::new(Self::VENDOR))
            .map(|_| Self::Vendored)
            .or_else(|| Platform::tests_covering(file).map(|_| Self::PlatformTests))
            .or_else(|| file.lies_under(Path::new(Self::ASSETS)).map(|_| Self::Assets))
            .or_else(|| file.lies_under(Path::new(Platform::NAME)).map(|_| Self::Platform))
            .unwrap_or(Self::Outside)
    }

    pub(crate) fn refusing(self, language: Language) -> Option<Language> {
        match self {
            Self::Outside | Self::Platform => Some(language),
            Self::Vendored | Self::PlatformTests => None,
            Self::Assets => (language.fix() != Fix::MoveToAssets).then_some(language),
        }
    }
}

impl fmt::Display for Place {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Outside => formatter.write_str("source outside assets"),
            Self::Vendored => formatter.write_str("vendored source"),
            Self::Assets => formatter.write_str("source below assets is not a web asset"),
            Self::PlatformTests => formatter.write_str("platform test source"),
            Self::Platform => formatter.write_str("source below platform is not test code"),
        }
    }
}
