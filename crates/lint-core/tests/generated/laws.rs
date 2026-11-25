use laws::proptest::prelude::any;

laws::generates!(aliases_generates, any::<lint_core::Aliases>());
laws::monoid!(aliases_forms_a_monoid, lint_core::Aliases, any::<lint_core::Aliases>());

laws::generates!(descend_generates, any::<lint_core::Descend>());
laws::monoid!(descend_forms_a_monoid, lint_core::Descend, any::<lint_core::Descend>());
