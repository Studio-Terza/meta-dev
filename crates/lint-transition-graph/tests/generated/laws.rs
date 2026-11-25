use laws::proptest::prelude::any;

laws::generates!(ask_generates, any::<lint_transition_graph::Ask>());

laws::generates!(listed_generates, any::<lint_transition_graph::Listed>());
