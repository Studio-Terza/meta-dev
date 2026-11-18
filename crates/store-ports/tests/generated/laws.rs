use laws::proptest::prelude::any;

laws::generates!(version_generates, any::<store_ports::Version>());

laws::generates!(written_generates, any::<store_ports::Written>());
