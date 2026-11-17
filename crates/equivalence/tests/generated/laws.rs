use laws::proptest::prelude::any;

laws::generates!(colour_generates, any::<equivalence::Colour>());

laws::generates!(holes_generates, any::<equivalence::Holes>());
