use laws::proptest::prelude::any;

laws::generates!(test_name_generates, any::<test_log::TestName>());
