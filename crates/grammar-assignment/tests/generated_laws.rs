use laws::proptest::prelude::any;

laws::generates!(emptied_generates, any::<grammar_assignment::Emptied>());
