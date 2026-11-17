use laws::proptest::prelude::any;

laws::generates!(cite_generates, any::<grammar_notes::Cite>());

laws::generates!(date_generates, any::<grammar_notes::Date>());

laws::generates!(quote_generates, any::<grammar_notes::Quote>());

laws::generates!(unclosed_generates, any::<grammar_notes::Unclosed>());
