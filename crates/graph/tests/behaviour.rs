use algebra::{Act, Semigroup};
use graph::{CrateRoots, External, ItemPath, Resolved, Root};
use laws::proptest::collection::vec;
use laws::proptest::prop_oneof;
use laws::proptest::sample::select;
use laws::proptest::strategy::{BoxedStrategy, Strategy};
use lint_layout::Name;
use lint_layout::PackageName;

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

#[test]
fn merged_crate_roots_answer_for_every_package_they_were_read_for() {
    let first = PackageName::from(&Name::from("first"));
    let second = PackageName::from(&Name::from("second"));
    let roots = CrateRoots::from_iter([(first.clone(), Root::Unread)])
        .combine(CrateRoots::from_iter([(second.clone(), Root::Unread)]));
    assert!(roots.root(&first).is_some());
    assert!(roots.root(&second).is_some());
    assert!(roots.root(&PackageName::from(&Name::from("third"))).is_none());
}
