# Stage 2c: headless workers, first piece

meta starts Claude sessions itself, under the rules in `docs/prereg.md`, "Headless workers". This first piece runs review leases only. The round cap and the token budget are deferred with token budgeting. The always-on supervisor is a separate decision, so nothing here touches launchd: a worker starts when someone runs `meta worker start`.

## The session

`meta worker start <lease>` runs this, with the brief as the prompt, and with `<allow>` and `<deny>` taken from `self/workers.toml`:

```
claude -p <brief> --tools <allow> --allowedTools <allow> --disallowedTools <deny> \
  --permission-mode dontAsk --permission-prompts none --strict-mcp-config \
  --output-format stream-json --verbose
```

Each flag, from the Claude Code docs (`/en/headless`, `/en/cli-reference`, `/en/permissions`, `/en/permission-modes`, `/en/tools-reference`) and `claude --help`, 2.1.314:

| Flag | Why |
|---|---|
| `-p <brief>` | Non-interactive. The prompt comes first because `--tools`, `--allowedTools` and `--disallowedTools` each take a list, which would swallow a prompt placed after them. |
| `--tools` | The session's tools are exactly these. Without it, `--allowedTools` only skips prompts and every other tool stays available. Naming `Glob` and `Grep` here also brings them back on macOS and Linux, where the default set leaves them out. |
| `--allowedTools` | These tools run with no approval. |
| `--disallowedTools` | Deny comes first in rule order, so nothing can allow these back. A bare name removes the tool. `--tools` does not cover MCP tools, so `mcp__*` is denied too. |
| `--permission-mode dontAsk` | Anything not allowed is denied, not asked. |
| `--permission-prompts none` | Nobody can answer, so Claude is told not to retry a denied call. |
| `--strict-mcp-config` | With no `--mcp-config`, no MCP server connects. |
| `--output-format stream-json --verbose` | The transcript is the whole run, event by event, and its last line has the cost. |

`--bare` is the recommended mode for scripts, but it reads no OAuth login and needs `ANTHROPIC_API_KEY`, which this machine's account does not use. Without it the session loads the user's settings, hooks and `CLAUDE.md`, as any `claude -p` does.

Each worker has a directory, `<data dir>/workers/worker-<n>/`, kept after it ends:

| Path | Holds |
|---|---|
| `transcript.jsonl` | The session's stdout and stderr |
| `scratch/` | The session's working directory, fresh: a directory already there refuses the start |
| `status` | The exit status, written by the session's own process group when `claude` exits |
| `supervisor.log` | The supervisor's output |

The session runs with `GH_REPO` set to the origin, because the scratch directory is no clone and the review brief needs `gh pr view` and `gh pr diff`. `meta verdict` needs it too.

## Supervision

`/bin/sh` leads the session's process group and is the parent of `claude`, because only a parent can read an exit status once `meta worker start` has returned. `meta worker start` spawns the session and then the supervisor, `meta worker wait <lease>`, each detached, and returns.

The session's standard input is a pipe whose other end only the supervisor holds. When that end closes, a process in the group kills the group. So if the supervisor dies for any reason, the worker dies with it, as the prereg asks: a worker never runs without a deadline.

The supervisor looks at its worker every 5 s, so it can be up to 5 s late at the deadline. It reads the log under the lock, so its first look waits for `start` to record the worker. A worker recorded by nothing therefore ends itself: the supervisor refuses the lease, exits, and takes the session with it.

Before it signals a group or calls a session alive, the port checks that the leader's command line names the run's transcript. A process id can be reused, and a stranger's group must never be killed.

## Behaviours

Each row is a behaviour-table test through the single in-process `Call`, with a recorded spawn port. No test runs `claude`. The port's own process handling is tested against `/bin/sh`.

| Command | Given | Result |
|---|---|---|
| `meta worker start <lease>` | a live, unused review lease with a deadline, under the limit | one session starts; events `LeaseUsed`, then `Started` with agent `worker-<n>`, the deadline counted from now, the process group and the transcript path; prints all of them and returns at once |
| `meta worker start <lease>` | the lease is unknown, used, not live, a work lease, or has no deadline | refused naming which; nothing starts; nothing is recorded |
| `meta worker start <lease>` | the running workers have reached `concurrency.running` in `self/workers.toml` | refused naming them, with `meta stop <lease>` as the next step. The check holds the log's lock, so two starts cannot both take the last place |
| `meta worker start <lease>` | a worker gave its verdict and its session has not ended | it still counts towards the limit |
| `meta worker start <lease>` | the log is real and the build is not from main | refused, as for any write; nothing starts |
| `meta worker start <lease>` | the log is real | `self/workers.toml` is read from main; the working tree's copy is ignored |
| `meta worker start <lease>` | the log is scratch | any build runs, with the working tree's `self/workers.toml`, so a branch can try the file it changes |
| `meta worker start <lease>` | `self/workers.toml` cannot be read, or an entry has no source | error; nothing starts; the lease stays unused |
| `meta worker start <lease>` | the session cannot be started | error; nothing is recorded; the lease stays unused |
| `meta worker wait <lease>` | the session ended and left an exit status | event `Finished` with `end: { exited: <status> }` |
| `meta worker wait <lease>` | the session is gone and left no exit status | event `Finished` with `end: vanished` |
| `meta worker wait <lease>` | the deadline passes while the session runs | the process group is killed by the port's group kill (SIGTERM, a short grace, SIGKILL); event `Finished` with `end: deadline` |
| `meta worker wait <lease>` | the task is waiting on the owner | the deadline holds all the same: a running session never goes without one |
| `meta worker wait <lease>` | the kill or the process table read fails | it keeps watching and tries again |
| `meta worker wait <lease>` | the log cannot be read after the first look | it keeps the last look, so the deadline still holds |
| `meta worker wait <lease>` | the worker's end is already recorded | exits with no event |
| `meta worker wait <lease>` | the lease started no worker | refused, which ends the session |
| `meta stop <lease>` | a worker is running on the lease | its process group is killed; event `Finished` with `end: stopped` |
| `meta stop` | any workers are running | every group is killed and each is recorded; with none running, says so |
| `meta stop <lease>` | the lease started no worker, or its worker has ended | refused; nothing is killed |
| `meta stop` | the build may not write the log | it kills all the same, and says nothing was recorded: the kill switch always kills. The supervisor then records `vanished` if it can |
| `meta stop` | a group cannot be killed | error naming it; the others are still killed and recorded |
| `meta status` | a worker's end is not recorded | "running agents" lists `<lease>: agent worker-<n>, <task>, role review, <deadline>, transcript <path>`, even if its task has ended, since the process is still there |
| `meta status` | a worker's end is recorded | not listed |
| `meta hook pre-tool-use` with tool `SendMessage` | the canned `continue` names a worker | denied: a headless session is not a subagent, and the worker's lease stays finished |

`meta stop` kills inside the log's lock, so its supervisor, waiting on that lock, finds the end recorded and does not call the session vanished.

## What this piece leaves open

- **Bash is not read-only.** `self/workers.toml` allows all of `Bash`, because the review brief needs `gh` and `meta verdict`. A reviewer can therefore write files and run any command, and a `Bash` deny rule such as `Bash(claude *)` stops only the form it matches (Claude Code permissions, Bash rule limits).
- **Secrets and paths outside the worktree are not denied yet.** The prereg asks for it. `Read` and `Bash` can reach both. Doing it properly means narrower Bash rules, or the Claude Code sandbox, which is a decision of its own.
- **A worker keeps the user's settings.** See `--bare` above.
- **A worker with no supervisor stays listed.** If the machine restarts, the sessions are gone but the log still shows them running, and they count towards the limit. `meta stop` clears them: the port sees that the leader is not theirs, kills nothing, and records the end.
- **A lost `Finished`.** If the log cannot be written when a worker ends, the supervisor reports it in `supervisor.log` and exits. `meta stop` clears that too.
