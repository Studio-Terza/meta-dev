use store_ports::{Execute, Query, StoreFailure};

use super::SessionEvents;
use crate::boundary::HeardEvent;

impl Query for SessionEvents {
    type Rows = Vec<HeardEvent>;

    fn rows(self, statements: &mut impl Execute) -> Result<Vec<HeardEvent>, StoreFailure> {
        let SessionEvents(session) = self;
        HeardEvent::in_session(&session, statements)
    }
}
