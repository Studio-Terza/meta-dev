use laws::proptest::prelude::any;

laws::generates!(term_generates, any::<grammar_search::Term>());
