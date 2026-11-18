use laws::proptest::prelude::any;

laws::generates!(heard_generates, any::<console_ports::Heard>());
