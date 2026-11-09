#![deny(clippy::wildcard_enum_match_arm)]

use grammar_cargo_option::{CargoOption, Options};

laws::table!(
    a_cargo_option_word_is_read_as_a_long_option_or_a_package_flag,
    |text: &&'static str| Options::spelled(text),
    [
        "--package" => CargoOption::Long { name: "package", value: None },
        "--package=a" => CargoOption::Long { name: "package", value: Some("a") },
        "--package=a=b" => CargoOption::Long { name: "package", value: Some("a=b") },
        "--package=" => CargoOption::Long { name: "package", value: Some("") },
        "--all-targets" => CargoOption::Long { name: "all-targets", value: None },
        "--packagex=a" => CargoOption::Long { name: "packagex", value: Some("a") },
        "--Package" => CargoOption::Plain,
        "--package_x" => CargoOption::Plain,
        "-p" => CargoOption::Package { value: None },
        "-pa" => CargoOption::Package { value: Some("a") },
        "-p=a" => CargoOption::Package { value: Some("a") },
        "-ppa" => CargoOption::Package { value: Some("pa") },
        "-vqp" => CargoOption::Package { value: None },
        "-vpa*" => CargoOption::Package { value: Some("a*") },
        "-v" => CargoOption::Plain,
        "-Fp" => CargoOption::Plain,
        "-j4" => CargoOption::Plain,
        "--" => CargoOption::Plain,
        "p" => CargoOption::Plain,
        "a-p" => CargoOption::Plain,
        "" => CargoOption::Plain,
    ]
);
