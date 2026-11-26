use laws::proptest::prelude::any;

laws::generates!(alias_generates, any::<graph::Alias>());

laws::generates!(alias_parameter_generates, any::<graph::AliasParameter>());

laws::generates!(aliases_generates, any::<graph::Aliases>());
laws::monoid!(aliases_forms_a_monoid, graph::Aliases, any::<graph::Aliases>());

laws::generates!(assigned_generates, any::<graph::Assigned>());

laws::generates!(call_generates, any::<graph::Call>());

laws::generates!(call_order_generates, any::<graph::CallOrder>());

laws::generates!(callee_generates, any::<graph::Callee>());

laws::generates!(crate_exports_generates, any::<graph::CrateExports>());
laws::monoid!(crate_exports_forms_a_monoid, graph::CrateExports, any::<graph::CrateExports>());

laws::generates!(crate_roots_generates, any::<graph::CrateRoots>());
laws::monoid!(crate_roots_forms_a_monoid, graph::CrateRoots, any::<graph::CrateRoots>());

laws::generates!(definition_generates, any::<graph::Definition>());

laws::generates!(dispatch_generates, any::<graph::Dispatch>());

laws::generates!(domain_generates, any::<graph::Domain>());

laws::generates!(entry_generates, any::<graph::Entry>());

laws::generates!(evaluation_generates, any::<graph::Evaluation>());

laws::generates!(exports_generates, any::<graph::Exports>());
laws::monoid!(exports_forms_a_monoid, graph::Exports, any::<graph::Exports>());

laws::generates!(exposure_generates, any::<graph::Exposure>());

laws::generates!(external_generates, any::<graph::External>());
laws::monoid!(external_forms_a_monoid, graph::External, any::<graph::External>());

laws::generates!(file_id_generates, any::<graph::FileId>());

laws::generates!(function_generates, any::<graph::Function>());

laws::generates!(ground_generates, any::<graph::Ground>());

laws::generates!(held_generates, any::<graph::Held>());

laws::generates!(holding_generates, any::<graph::Holding>());

laws::generates!(implementation_generates, any::<graph::Implementation>());

laws::generates!(infix_generates, any::<graph::Infix>());

laws::generates!(input_generates, any::<graph::Input>());

laws::generates!(interface_generates, any::<graph::Interface>());

laws::generates!(invocation_generates, any::<graph::Invocation>());

laws::generates!(item_declaration_generates, any::<graph::ItemDeclaration>());

laws::generates!(item_id_generates, any::<graph::ItemId>());

laws::generates!(item_path_generates, any::<graph::ItemPath>());
laws::monoid!(item_path_forms_a_monoid, graph::ItemPath, any::<graph::ItemPath>());

laws::generates!(law_generates, any::<graph::Law>());

laws::generates!(law_kind_generates, any::<graph::LawKind>());

laws::generates!(literal_value_generates, any::<graph::LiteralValue>());

laws::generates!(looping_generates, any::<graph::Looping>());

laws::generates!(macro_call_generates, any::<graph::MacroCall>());

laws::generates!(member_generates, any::<graph::Member>());

laws::generates!(mention_generates, any::<graph::Mention>());

laws::generates!(method_shape_generates, any::<graph::MethodShape>());

laws::generates!(model_part_generates, any::<graph::ModelPart>());
laws::monoid!(model_part_forms_a_monoid, graph::ModelPart, any::<graph::ModelPart>());

laws::generates!(mutation_generates, any::<graph::Mutation>());

laws::generates!(operation_generates, any::<graph::Operation>());

laws::generates!(operation_owner_generates, any::<graph::OperationOwner>());

laws::generates!(operator_generates, any::<graph::Operator>());

laws::generates!(origin_generates, any::<graph::Origin>());

laws::generates!(output_generates, any::<graph::Output>());

laws::generates!(owner_generates, any::<graph::Owner>());

laws::generates!(param_index_generates, any::<graph::ParamIndex>());

laws::generates!(parameter_generates, any::<graph::Parameter>());

laws::generates!(path_use_generates, any::<graph::PathUse>());

laws::generates!(prefix_generates, any::<graph::Prefix>());

laws::generates!(primitive_generates, any::<graph::Primitive>());

laws::generates!(property_generates, any::<graph::Property>());

laws::generates!(quote_generates, any::<graph::Quote>());

laws::generates!(quoted_generates, any::<graph::Quoted>());

laws::generates!(reach_generates, any::<graph::Reach>());

laws::generates!(reached_generates, any::<graph::Reached>());

laws::generates!(read_generates, any::<graph::Read>());

laws::generates!(reader_generates, any::<graph::Reader>());

laws::generates!(receiver_generates, any::<graph::Receiver>());

laws::generates!(receiving_generates, any::<graph::Receiving>());

laws::generates!(reexport_generates, any::<graph::Reexport>());

laws::generates!(references_generates, any::<graph::References>());
laws::monoid!(references_forms_a_monoid, graph::References, any::<graph::References>());

laws::generates!(region_generates, any::<graph::Region>());
laws::monoid!(region_forms_a_monoid, graph::Region, any::<graph::Region>());

laws::generates!(repeating_generates, any::<graph::Repeating>());

laws::generates!(repetition_generates, any::<graph::Repetition>());

laws::generates!(resolved_generates, any::<graph::Resolved>());

laws::generates!(returned_generates, any::<graph::Returned>());

laws::generates!(root_generates, any::<graph::Root>());

laws::generates!(route_generates, any::<graph::Route>());

laws::generates!(seat_generates, any::<graph::Seat>());

laws::generates!(side_generates, any::<graph::Side>());

laws::generates!(signature_generates, any::<graph::Signature>());

laws::generates!(sited_generates, any::<graph::Sited>());

laws::generates!(spelling_generates, any::<graph::Spelling>());

laws::generates!(subject_generates, any::<graph::Subject>());

laws::generates!(taken_generates, any::<graph::Taken>());

laws::generates!(unresolved_generates, any::<graph::Unresolved>());

laws::generates!(within_generates, any::<graph::Within>());

laws::generates!(writing_generates, any::<graph::Writing>());

laws::generates!(zone_generates, any::<graph::Zone>());
