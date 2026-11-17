use laws::proptest::prelude::any;

laws::generates!(token_count_generates, any::<body_shape::TokenCount>());
