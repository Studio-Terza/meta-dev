use store_ports::{Query, Statements, StoreFailure};

use super::SessionEvents;
use crate::boundary::HeardEvent;

impl Query for SessionEvents {
    type Rows = Vec<HeardEvent>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Vec<HeardEvent>, StoreFailure> {
        let SessionEvents(session) = self;
        HeardEvent::in_session(&session, statements)
    }
}
