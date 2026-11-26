use laws::proptest::prelude::any;

laws::generates!(program_generates, any::<grammar_program::Program>());
