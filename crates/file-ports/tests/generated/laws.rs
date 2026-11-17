use laws::proptest::prelude::any;

laws::generates!(bytes_generates, any::<file_ports::Bytes>());

laws::generates!(offset_generates, any::<file_ports::Offset>());

laws::generates!(size_generates, any::<file_ports::Size>());

laws::generates!(text_generates, any::<file_ports::Text>());
