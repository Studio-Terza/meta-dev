use laws::proptest::prelude::any;

laws::generates!(banner_generates, any::<grammar_banner::Banner>());
