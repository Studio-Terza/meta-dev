use laws::proptest::prelude::any;

laws::generates!(length_generates, any::<lint_layout::Length>());

laws::generates!(package_name_generates, any::<lint_layout::PackageName>());

laws::generates!(path_dependencies_generates, any::<lint_layout::PathDependencies>());

laws::generates!(source_text_generates, any::<lint_layout::SourceText>());

laws::generates!(untested_generates, any::<lint_layout::Untested>());
