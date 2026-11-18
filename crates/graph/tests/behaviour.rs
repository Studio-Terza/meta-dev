use algebra::Act;
use graph::{External, ItemPath, Resolved};
use laws::proptest::collection::vec;
use laws::proptest::prop_oneof;
use laws::proptest::sample::select;
use laws::proptest::strategy::{BoxedStrategy, Strategy};
use lint_layout::Name;

struct Drawn;

impl Drawn {
    fn paths() -> BoxedStrategy<ItemPath> {
        vec(select(vec!["std", "fmt", "a"]).prop_map(Name::from), 0..3)
            .prop_map(|names| names.into_iter().collect::<ItemPath>())
            .boxed()
    }

    fn resolutions() -> BoxedStrategy<Resolved> {
        prop_oneof![
            Drawn::paths().prop_map(Resolved::Std),
            Drawn::paths().prop_map(Resolved::StdTrait),
            Drawn::paths().prop_map(|path| Resolved::External(External::new(None, path))),
            select(vec!["T", "U"]).prop_map(|name| Resolved::Param(Name::from(name))),
        ]
        .boxed()
    }
}

laws::action!(a_path_acts_on_a_resolution, Drawn::resolutions(), <Resolved as Act<ItemPath>>::acted, Drawn::paths());
