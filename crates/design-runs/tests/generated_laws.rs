use laws::proptest::prelude::any;

laws::generates!(account_generates, any::<design_runs::Account>());

laws::generates!(excerpt_generates, any::<design_runs::Excerpt>());
