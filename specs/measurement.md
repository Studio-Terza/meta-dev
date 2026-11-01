# measurement

Problem: slow code and creep must show up early, without enumerating every cause.

L1 Wall time of the test suite and CI steps is recorded on every main run.
L2 Blocking limits are set by the owner, never chosen by an agent.
L3 Counts are deterministic: the same on any machine.
D1 CI prints a report on main: wall time per step, tests, test lines and process starts per crate.
D2 Lint: process starts are counted through the `ports` convention and reported per crate. after: D1
D3 Wall-time limits with warn and block levels. after: D1
D4 Local token report: spend per day, model, lead and agents, from transcripts. Kept outside this repository.
D5 One tracking table per project: details done, spend, attached share, features delivered. after: D4
Out: runner noise, per-PR judging.
