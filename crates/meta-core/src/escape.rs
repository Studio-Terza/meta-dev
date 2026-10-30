use crate::event::{Event, Record};
use crate::ids::Reason;
use crate::reply::Accepted;

#[derive(Debug)]
pub struct Escape(Reason);

impl Escape {
    pub fn new(reason: Reason) -> Self {
        Self(reason)
    }

    pub fn tally(self, earlier: Result<&[Record], &str>) -> Accepted {
        let reason = self.0;
        let counts = earlier.map(|records| {
            records.iter().fold((1, 0), |(open, repeats), record| match &record.event {
                Event::Escaped { reason: earlier } => (open + 1, repeats + usize::from(*earlier == reason)),
                _ => (open, repeats),
            })
        });
        let tally = match counts {
            Ok((open, repeats)) => format!("open escapes: {open}\nthis reason repeated: {repeats}\n"),
            Err(why) => format!("counts unavailable: {why}\n"),
        };
        let text = format!("escape recorded: {reason}\n{tally}");
        Accepted::new(vec![Event::Escaped { reason }], text)
    }
}
