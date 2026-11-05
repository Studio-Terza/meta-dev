use lint_core::Placeholders;

struct Arguments;

impl Arguments {
    fn of(text: &str) -> Vec<String> {
        text.parse::<Placeholders>()
            .map(Placeholders::streams)
            .unwrap_or_default()
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    fn spelled(text: &str) -> String {
        text.parse::<proc_macro2::TokenStream>().map(|stream| stream.to_string()).unwrap_or_default()
    }
}

#[test]
fn a_path_separator_is_not_the_start_of_a_format_spec() {
    assert_eq!(Arguments::of("{std::mem::size_of::<u8>()}"), [Arguments::spelled("std::mem::size_of::<u8>()")]);
}

#[test]
fn a_format_spec_starts_at_the_first_lone_colon() {
    assert_eq!(Arguments::of("{a:b:c}"), [Arguments::spelled("a")]);
}

#[test]
fn a_lone_colon_after_a_path_separator_starts_the_spec() {
    assert_eq!(Arguments::of("{a::b:>5}"), [Arguments::spelled("a::b")]);
}
