use lint_core::Parsed;
use lint_file::ContextLint;
use lint_named_numbers::NamedNumbers;

pub(crate) trait Holding<'run, Lint: ContextLint> {
    fn held(&self) -> &Lint::Context<'run>;
}

macro_rules! contexts {
    ($($field:ident: $lint:ty),* $(,)?) => {
        #[derive(Debug)]
        pub(crate) struct Contexts<'run> {
            $($field: <$lint as ContextLint>::Context<'run>,)*
        }

        #[derive(Debug, Default)]
        struct Parts<'run> {
            $($field: Vec<<$lint as ContextLint>::Context<'run>>,)*
        }

        impl<'run> From<Parsed<'_, 'run>> for Contexts<'run> {
            fn from(parsed: Parsed<'_, 'run>) -> Contexts<'run> {
                Contexts { $($field: <$lint as ContextLint>::contributed(parsed),)* }
            }
        }

        impl<'run> FromIterator<Contexts<'run>> for Contexts<'run> {
            fn from_iter<Found: IntoIterator<Item = Contexts<'run>>>(found: Found) -> Contexts<'run> {
                let split = found.into_iter().fold(Parts::default(), |mut split, part| {
                    $(split.$field.push(part.$field);)*
                    split
                });
                Contexts { $($field: split.$field.into_iter().collect(),)* }
            }
        }

        $(
            impl<'run> Holding<'run, $lint> for Contexts<'run> {
                fn held(&self) -> &<$lint as ContextLint>::Context<'run> {
                    &self.$field
                }
            }
        )*
    };
}

contexts! {
    named_numbers: NamedNumbers,
}
