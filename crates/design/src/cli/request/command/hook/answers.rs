mod boundary;

use design_messages::{Quote, Said};
use dispatch_ports::Timeout;
use transcript_ports::Body;

use super::super::crew::{Asked, Prompt, Tape};
use boundary::Asked as Question;
pub use boundary::{Answers, Moment};

impl Answers {
    const TIMEOUT: Timeout = Timeout::new(120);

    pub fn searched(tape: &Tape<'_>, said: &Body, moment: Moment, history: &[Said]) -> Result<Option<Answers>, Asked> {
        let asked = Question::new(said, moment);
        let reading = asked.reading(history);
        let fanned = tape.fanned(reading.prompts(), Self::TIMEOUT)?;
        let quoted: Vec<Quote> = reading.quoted(&fanned).into_iter().flatten().collect();
        let trimmed = match quoted.first() {
            Some(_first) => tape.replied(&Prompt::from(&asked.trimming(&quoted)), Self::TIMEOUT)?,
            None => None,
        };
        Ok(trimmed.and_then(|reply| Answers::kept(&reply, &quoted)))
    }
}
