use laws::proptest::prelude::any;

laws::generates!(tally_generates, any::<design_messages::Tally>());
