use laws::proptest::prelude::any;

laws::generates!(applied_id_generates, any::<design_graph::AppliedId>());
