# resources

Problem: cleanup and lifecycles are done by hand, so resources leak and paths skip their ending.

L1 Raw resource calls happen only behind a guard in `ports`.
L2 Every path through a declared lifecycle reaches a terminal state, checked at compile time on stable Rust.
L3 No allow-lists.
D1 Ban raw `Command::spawn`, `fs::remove_dir_all`, `mem::forget`, `env::temp_dir` outside `ports`, through clippy `disallowed-methods` with per-crate configuration or the existing `ports` convention, whichever is simpler.
D2 Lint: paired APIs such as start/stop, lock/unlock, open/close, claim/release are reported, with a message saying to write a guard. Report only.
D3 `#[must_use]` required on guard types.
D4 `states!` macro: declares a graph of states and transitions and generates state types, transitions and `#[must_use]`. Lifecycles use witness tokens: a branded return value that only a terminal transition can construct, so a function that drops a state cannot compile. after: D3
D5 Lint: every state in a `states!` graph reaches a terminal, nothing is unreachable, terminals have no outgoing edges. after: D4
Out: agent-level cleanup, async cancellation, drop bombs, nightly-only tooling.
