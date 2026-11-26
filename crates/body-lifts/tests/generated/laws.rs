use laws::proptest::prelude::any;

laws::generates!(body_generates, any::<body_lifts::Body>());

laws::generates!(branching_generates, any::<body_lifts::Branching>());

laws::generates!(propagated_generates, any::<body_lifts::Propagated>());
