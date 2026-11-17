use laws::proptest::prelude::any;

laws::generates!(branching_generates, any::<body_lifts::Branching>());

laws::generates!(propagated_generates, any::<body_lifts::Propagated>());
