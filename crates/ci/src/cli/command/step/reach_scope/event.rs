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
        if name == Self::PULL_REQUEST { Event::PullRequest } else { Event::Other }
    }
}
