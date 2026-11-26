use laws::proptest::prelude::any;

laws::generates!(keyed_generates, any::<design_closure::Keyed>());

laws::generates!(listed_generates, any::<design_closure::Listed>());
