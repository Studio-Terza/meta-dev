use laws::proptest::prelude::any;

laws::generates!(number_generates, any::<grammar_message::Number>());
