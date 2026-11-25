use laws::proptest::prelude::any;

laws::generates!(number_generates, any::<grammar_message::Number>());

laws::generates!(span_generates, any::<grammar_message::Span>());
laws::semigroup!(span_forms_a_semigroup, grammar_message::Span, any::<grammar_message::Span>());
