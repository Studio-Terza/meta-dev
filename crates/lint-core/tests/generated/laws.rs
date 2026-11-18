use laws::proptest::prelude::any;

laws::generates!(descend_generates, any::<lint_core::Descend>());
laws::monoid!(descend_forms_a_monoid, lint_core::Descend, any::<lint_core::Descend>());
