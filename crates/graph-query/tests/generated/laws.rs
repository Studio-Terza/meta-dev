use laws::proptest::prelude::any;

laws::generates!(query_generates, any::<graph_query::Query>());
laws::monoid!(query_forms_a_monoid, graph_query::Query, any::<graph_query::Query>());

laws::generates!(role_generates, any::<graph_query::Role>());

laws::generates!(term_generates, any::<graph_query::Term>());
