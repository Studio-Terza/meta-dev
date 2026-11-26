use laws::proptest::prelude::any;

laws::generates!(crate_name_generates, any::<design_model::CrateName>());

laws::generates!(feature_name_generates, any::<design_model::FeatureName>());

laws::generates!(job_id_generates, any::<design_model::JobId>());

laws::generates!(node_id_generates, any::<design_model::NodeId>());

laws::generates!(notes_line_generates, any::<design_model::NotesLine>());

laws::generates!(ordinal_generates, any::<design_model::Ordinal>());

laws::generates!(prefix_generates, any::<design_model::Prefix>());

laws::generates!(project_generates, any::<design_model::Project>());

laws::generates!(run_id_generates, any::<design_model::RunId>());

laws::generates!(title_generates, any::<design_model::Title>());
