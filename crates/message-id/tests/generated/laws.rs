use laws::proptest::prelude::any;

laws::generates!(uuid_generates, any::<message_id::Uuid>());
