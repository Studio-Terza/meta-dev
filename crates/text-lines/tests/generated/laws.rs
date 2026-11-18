use laws::proptest::prelude::any;

laws::generates!(one_line_generates, any::<text_lines::OneLine>());
