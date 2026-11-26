use laws::proptest::prelude::any;

laws::generates!(ideal_id_generates, any::<design_messages::IdealId>());

laws::generates!(line_number_generates, any::<design_messages::LineNumber>());

laws::generates!(revision_generates, any::<design_messages::Revision>());

laws::generates!(source_path_generates, any::<design_messages::SourcePath>());

laws::generates!(tally_generates, any::<design_messages::Tally>());
