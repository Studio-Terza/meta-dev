use laws::proptest::prelude::any;

laws::generates!(count_generates, any::<design::Count>());
