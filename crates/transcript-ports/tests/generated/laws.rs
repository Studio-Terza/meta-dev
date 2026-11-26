use laws::proptest::prelude::any;

laws::generates!(body_generates, any::<transcript_ports::Body>());

laws::generates!(cursor_generates, any::<transcript_ports::Cursor>());

laws::generates!(instant_generates, any::<transcript_ports::Instant>());

laws::generates!(kind_generates, any::<transcript_ports::Kind>());

laws::generates!(session_id_generates, any::<transcript_ports::SessionId>());
