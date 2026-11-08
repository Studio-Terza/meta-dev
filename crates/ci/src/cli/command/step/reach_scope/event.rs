#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    PullRequest,
    Other,
}

impl Event {
    const PULL_REQUEST: &'static str = "pull_request";
}

impl From<&str> for Event {
    fn from(name: &str) -> Event {
        Some(name).filter(|candidate| *candidate == Self::PULL_REQUEST).map_or(Event::Other, |_| Event::PullRequest)
    }
}
