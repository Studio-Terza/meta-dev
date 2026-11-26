use laws::proptest::prelude::any;

laws::generates!(attachment_generates, any::<grammar_spec_line::Attachment>());
