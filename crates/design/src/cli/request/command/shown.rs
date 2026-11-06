use std::fmt;

use port_core::Metered;
use stream::{MechanicalLink, Message, Shape, Stream};
use transcript_ports::{Transcripts, Uuid};

use crate::cli::failure::Failure;

#[derive(Debug)]
pub struct Shown {
    message: Message,
    links: Vec<MechanicalLink>,
}

impl Shown {
    pub fn find(port: &(impl Transcripts + Metered), uuid: &Uuid) -> Result<Shown, Failure> {
        let sessions = port.sessions()?;
        port.read_all(&sessions)?
            .into_iter()
            .map(|read| read.into_iter().collect::<Stream>())
            .find_map(|stream| Self::within(&stream, uuid))
            .ok_or_else(|| Failure::Missing(uuid.clone()))
    }

    fn within(stream: &Stream, uuid: &Uuid) -> Option<Shown> {
        stream
            .as_ref()
            .iter()
            .find(|message| message.record().uuid() == uuid)
            .map(|message| Shown { message: message.clone(), links: Self::linked(stream, uuid) })
    }

    fn linked(stream: &Stream, uuid: &Uuid) -> Vec<MechanicalLink> {
        stream.links().into_iter().filter(|link| link.touching(uuid).is_some()).collect()
    }

    fn segment_lines(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.segments().iter().try_for_each(|segment| {
            writeln!(
                formatter,
                "  {} {}: {:?}",
                segment.identity().ordinal(),
                Shape::from(segment.form()),
                segment.text(&self.message).to_string()
            )
        })
    }
}

impl fmt::Display for Shown {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let record = self.message.record();
        writeln!(
            formatter,
            "{} {} session {} at {}",
            record.uuid(),
            record.author(),
            record.session(),
            record.stamp()
        )?;
        self.segment_lines(formatter)?;
        self.links.iter().try_for_each(|link| writeln!(formatter, "  link {link}"))
    }
}
