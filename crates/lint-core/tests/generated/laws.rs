use laws::proptest::prelude::any;

laws::generates!(aliases_generates, any::<lint_core::Aliases>());
laws::monoid!(aliases_forms_a_monoid, lint_core::Aliases, any::<lint_core::Aliases>());

laws::generates!(descend_generates, any::<lint_core::Descend>());
laws::monoid!(descend_forms_a_monoid, lint_core::Descend, any::<lint_core::Descend>());

laws::generates!(visits_generates, any::<lint_core::Visits>());
laws::monoid!(visits_forms_a_monoid, lint_core::Visits, any::<lint_core::Visits>());

laws::generates!(walk_position_generates, any::<lint_core::WalkPosition>());
laws::monoid!(walk_position_forms_a_monoid, lint_core::WalkPosition, any::<lint_core::WalkPosition>());
