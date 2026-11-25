use laws::proptest::prelude::any;

laws::generates!(crate_exports_generates, any::<graph::CrateExports>());
laws::monoid!(crate_exports_forms_a_monoid, graph::CrateExports, any::<graph::CrateExports>());

laws::generates!(crate_roots_generates, any::<graph::CrateRoots>());
laws::monoid!(crate_roots_forms_a_monoid, graph::CrateRoots, any::<graph::CrateRoots>());

laws::generates!(domain_generates, any::<graph::Domain>());

laws::generates!(exports_generates, any::<graph::Exports>());
laws::monoid!(exports_forms_a_monoid, graph::Exports, any::<graph::Exports>());

laws::generates!(external_generates, any::<graph::External>());
laws::monoid!(external_forms_a_monoid, graph::External, any::<graph::External>());

laws::generates!(ground_generates, any::<graph::Ground>());

laws::generates!(item_id_generates, any::<graph::ItemId>());

laws::generates!(item_path_generates, any::<graph::ItemPath>());
laws::monoid!(item_path_forms_a_monoid, graph::ItemPath, any::<graph::ItemPath>());

laws::generates!(member_generates, any::<graph::Member>());

laws::generates!(method_shape_generates, any::<graph::MethodShape>());

laws::generates!(primitive_generates, any::<graph::Primitive>());

laws::generates!(reach_generates, any::<graph::Reach>());

laws::generates!(receiver_generates, any::<graph::Receiver>());

laws::generates!(reexport_generates, any::<graph::Reexport>());

laws::generates!(references_generates, any::<graph::References>());
laws::monoid!(references_forms_a_monoid, graph::References, any::<graph::References>());

laws::generates!(region_generates, any::<graph::Region>());
laws::monoid!(region_forms_a_monoid, graph::Region, any::<graph::Region>());

laws::generates!(resolved_generates, any::<graph::Resolved>());

laws::generates!(returned_generates, any::<graph::Returned>());

laws::generates!(root_generates, any::<graph::Root>());

laws::generates!(taken_generates, any::<graph::Taken>());

laws::generates!(unresolved_generates, any::<graph::Unresolved>());

laws::generates!(within_generates, any::<graph::Within>());

laws::generates!(writing_generates, any::<graph::Writing>());

laws::generates!(zone_generates, any::<graph::Zone>());
