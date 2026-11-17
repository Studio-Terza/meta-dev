use laws::proptest::prelude::any;

laws::generates!(listed_generates, any::<lint_transition_graph::Listed>());
