use laws::proptest::prelude::any;

laws::generates!(external_generates, any::<graph::External>());
laws::monoid!(external_forms_a_monoid, graph::External, any::<graph::External>());

laws::generates!(ground_generates, any::<graph::Ground>());

laws::generates!(item_id_generates, any::<graph::ItemId>());

laws::generates!(item_path_generates, any::<graph::ItemPath>());
laws::monoid!(item_path_forms_a_monoid, graph::ItemPath, any::<graph::ItemPath>());

laws::generates!(reach_generates, any::<graph::Reach>());

laws::generates!(references_generates, any::<graph::References>());
laws::monoid!(references_forms_a_monoid, graph::References, any::<graph::References>());

laws::generates!(region_generates, any::<graph::Region>());
laws::monoid!(region_forms_a_monoid, graph::Region, any::<graph::Region>());

laws::generates!(within_generates, any::<graph::Within>());

laws::generates!(writing_generates, any::<graph::Writing>());

laws::generates!(zone_generates, any::<graph::Zone>());
