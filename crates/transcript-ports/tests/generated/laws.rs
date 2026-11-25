use laws::proptest::prelude::any;

laws::generates!(authorship_generates, any::<transcript_ports::Authorship>());

laws::generates!(body_generates, any::<transcript_ports::Body>());

laws::generates!(cursor_generates, any::<transcript_ports::Cursor>());

laws::generates!(instant_generates, any::<transcript_ports::Instant>());

laws::generates!(kind_generates, any::<transcript_ports::Kind>());

laws::generates!(record_generates, any::<transcript_ports::Record>());

laws::generates!(session_id_generates, any::<transcript_ports::SessionId>());

laws::generates!(skipped_generates, any::<transcript_ports::Skipped>());

laws::generates!(unrecognised_generates, any::<transcript_ports::Unrecognised>());
