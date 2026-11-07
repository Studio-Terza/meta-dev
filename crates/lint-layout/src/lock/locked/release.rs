use std::convert::Infallible;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Release {
    version: Option<Version>,
    origin: Option<Origin>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Version(String);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Origin(String);

impl Release {
    pub(super) fn new(version: Option<Version>, origin: Option<Origin>) -> Release {
        Release { version, origin }
    }

    pub(super) fn shared(&self, other: &Release) -> Option<&Release> {
        Some(self).filter(|release| *release == other)
    }

    pub(super) fn versioned(&self, version: Option<&Version>) -> Option<&Release> {
        Some(self).filter(|release| version.is_none_or(|version| release.version.as_ref() == Some(version)))
    }
}

impl FromStr for Version {
    type Err = Infallible;

    fn from_str(spelled: &str) -> Result<Version, Infallible> {
        Ok(Version(spelled.to_owned()))
    }
}

impl FromStr for Origin {
    type Err = Infallible;

    fn from_str(spelled: &str) -> Result<Origin, Infallible> {
        Ok(Origin(spelled.to_owned()))
    }
}
