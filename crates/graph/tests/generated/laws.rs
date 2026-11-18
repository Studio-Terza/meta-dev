use laws::proptest::prelude::any;

laws::generates!(item_id_generates, any::<graph::ItemId>());

laws::generates!(references_generates, any::<graph::References>());
laws::monoid!(references_forms_a_monoid, graph::References, any::<graph::References>());
