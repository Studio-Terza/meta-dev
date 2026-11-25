use laws::proptest::prelude::any;

laws::generates!(message_generates, any::<stream::Message>());

laws::generates!(stream_generates, any::<stream::Stream>());
laws::semigroup!(stream_forms_a_semigroup, stream::Stream, any::<stream::Stream>());
