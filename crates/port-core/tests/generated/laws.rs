use laws::proptest::prelude::any;

laws::generates!(calls_generates, any::<port_core::Calls>());
