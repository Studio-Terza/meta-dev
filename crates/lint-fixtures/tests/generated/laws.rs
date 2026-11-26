use laws::proptest::prelude::any;

laws::generates!(lines_generates, any::<lint_fixtures::Lines>());
