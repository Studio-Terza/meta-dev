#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    PullRequest,
    Other,
}

impl Event {
    const PULL_REQUEST: &'static str = "pull_request";
}

impl From<&str> for Event {
    fn from(name: &str) -> Self {
        Some(name).filter(|name| *name == Self::PULL_REQUEST).map_or(Self::Other, |_| Self::PullRequest)
    }
}
