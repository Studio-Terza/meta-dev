use laws::proptest::prelude::any;

laws::generates!(crate_name_generates, any::<workspace_metadata::CrateName>());

laws::generates!(outside_generates, any::<workspace_metadata::Outside>());
