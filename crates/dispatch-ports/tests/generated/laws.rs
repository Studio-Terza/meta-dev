use laws::proptest::prelude::any;

laws::generates!(deadline_generates, any::<dispatch_ports::Deadline>());

laws::generates!(fault_generates, any::<dispatch_ports::Fault>());

laws::generates!(pause_generates, any::<dispatch_ports::Pause>());

laws::generates!(pid_generates, any::<dispatch_ports::Pid>());

laws::generates!(prompt_generates, any::<dispatch_ports::Prompt>());

laws::generates!(reply_generates, any::<dispatch_ports::Reply>());

laws::generates!(timeout_generates, any::<dispatch_ports::Timeout>());

laws::generates!(tokens_generates, any::<dispatch_ports::Tokens>());

laws::generates!(told_generates, any::<dispatch_ports::Told>());

laws::generates!(wall_generates, any::<dispatch_ports::Wall>());
