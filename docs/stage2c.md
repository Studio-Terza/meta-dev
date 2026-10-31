# Stage 2c: headless workers, first piece

meta starts Claude sessions itself, under the rules in `docs/prereg.md`, "Headless workers". This first piece runs review leases only. The round cap and the token budget are deferred with token budgeting. The always-on supervisor is a separate decision, so nothing here touches launchd: a worker starts when someone runs `meta worker start`.

## The session

`meta worker start <lease>` runs this, with the brief as the prompt, and with `<allow>` and `<deny>` taken from `self/workers.toml`:

```
claude -p <brief> --restricted --tools <allow> --allowedTools <allow> --disallowedTools <deny> \
  --json-schema <verdict schema> --no-session-persistence \
  --permission-mode dontAsk --permission-prompts none --strict-mcp-config \
  --output-format stream-json --verbose
```

Today `<allow>` is `Read,Grep,Glob` and `<deny>` is `Agent,Task,Bash,mcp__*`. The session has no way to run a command. Each flag, from the Claude Code docs (`/en/headless`, `/en/cli-reference`, `/en/permissions`, `/en/permission-modes`, `/en/tools-reference`, `/en/agent-sdk/structured-outputs`) and `claude --help`, 2.1.314:

| Flag | Why |
|---|---|
| `-p <brief>` | Non-interactive. The prompt comes first because `--tools`, `--allowedTools` and `--disallowedTools` each take a list, which would swallow a prompt placed after them. |
| `--restricted` | Removes the tools that run commands or code, confines the file tools to the working directories, loads only managed settings and `--settings` (so no user or project hooks or permission rules), and refuses `bypassPermissions`. Unlike `--bare`, it keeps the subscription login. |
| `--tools` | The session's tools are exactly these. Without it, `--allowedTools` only skips prompts and every other tool stays available. Naming `Glob` and `Grep` here also brings them back on macOS and Linux, where the default set leaves them out. |
| `--allowedTools` | These tools run with no approval. |
| `--disallowedTools` | Deny comes first in rule order, so nothing can allow these back. A bare name removes the tool. `--tools` does not cover MCP tools, so `mcp__*` is denied too. |
| `--json-schema` | The session must end with `{verdict: pass or fail, findings: [text]}`. It adds the `StructuredOutput` tool, whose input is this verdict. |
| `--no-session-persistence` | Nothing to resume: the transcript below is the record. |
| `--permission-mode dontAsk` | Anything not allowed is denied, not asked. |
| `--permission-prompts none` | Nobody can answer, so Claude is told not to retry a denied call. |
| `--strict-mcp-config` | With no `--mcp-config`, no MCP server connects. |
| `--output-format stream-json --verbose` | The transcript is the whole run, event by event, and its last event has the cost and the structured output. |

**Meta does the trusted work.** The session has no `gh`, no token and no `meta`. Before it starts, `meta worker start` fetches `gh pr view` and `gh pr diff`, and then checks that the PR still stands at the lease's head, so a head that moved during the fetch refuses the start. When the session ends, `meta worker wait` takes the verdict from the transcript and records it through the same check as `meta verdict`.

**Environment.** The session and the `claude --version` call start from an empty environment plus `PATH`, `HOME` and `USER`. A subscription login needs nothing else: a probe under `env -i` with those three logged in and ran. So `GH_TOKEN`, `SSH_AUTH_SOCK`, `CLAUDE_CODE_MESSAGING_TOKEN` and the like never reach a session. The supervisor keeps the caller's environment, plus `GH_REPO` set to the origin, because it runs in the worker's directory, which is no clone, and it reads GitHub for the verdict.

**Version floor.** `[claude] floor` in `self/workers.toml` is the oldest `claude` a worker runs on. Below it the start is refused; a reply that does not parse is an error. That `claude --version` is the one call to `claude` besides the sessions. It is 2.1.314, the version the flags were probed on. `--restricted` itself needs 2.1.248.

Each worker has a directory, `<data dir>/workers/worker-<n>/`, mode 0700, kept after it ends:

| Path | Mode | Holds |
|---|---|---|
| `transcript.jsonl` | 0600 | The session's stdout and stderr |
| `scratch/` | 0700 | The session's working directory, fresh |
| `scratch/pr-view.json` | 0600 | `gh pr view` of the PR |
| `scratch/pr.diff` | 0600 | `gh pr diff` of the PR |
| `run.json` | 0600 | The process group and transcript path, so that `meta stop` finds the session without the log |
| `status` | 0600 | The exit status, written by the session's own process group when `claude` exits |
| `supervisor.log` | 0600 | The supervisor's output: how the worker ended, and the verdict with its findings or why there is none |

A directory already there is what a failed start left, because the lease is still unused. It moves to `<data dir>/workers/failed/worker-<n>.<attempt>`, kept, and the start goes on. If its session still runs, the start is refused.

## Supervision

`/bin/sh` leads the session's process group and is the parent of `claude`, because only a parent can read an exit status once `meta worker start` has returned. `meta worker start` spawns the session and then the supervisor, `meta worker wait <lease>`, each detached, and returns.

The session's standard input is a pipe whose other end only the supervisor holds. When that end closes, a process in the group kills the group. So if the supervisor dies for any reason, the worker dies with it, as the prereg asks: a worker never runs without a deadline.

The supervisor looks at its worker every 5 s, so it can be up to 5 s late at the deadline. It reads the log under the lock, so its first look waits for `start` to record the worker. A worker recorded by nothing therefore ends itself: the supervisor refuses the lease, exits, and takes the session with it.

Before it signals a group or calls a session alive, the port checks that the leader's command line names the run's transcript. A process id can be reused, and a stranger's group must never be killed.

## Behaviours

Each row is a behaviour-table test through the single in-process `Call`, with a recorded spawn port. No test runs `claude`. The port's own process handling is tested against `/bin/sh`.

| Command | Given | Result |
|---|---|---|
| `meta worker start <lease>` | a live, unused review lease with a deadline, under the limit | one session starts, with the PR's view and diff in its scratch directory; events `LeaseUsed`, then `Started` with agent `worker-<n>`, the deadline counted from now, the process group and the transcript path; prints all of them and returns at once |
| `meta worker start <lease>` | the lease is unknown, used, not live, a work lease, or has no deadline | refused naming which; nothing starts; nothing is recorded |
| `meta worker start <lease>` | the running workers have reached `concurrency.running` in `self/workers.toml` | refused naming them, with `meta stop <lease>` as the next step. The check holds the log's lock, so two starts cannot both take the last place. The limit is per log: a scratch log counts only its own workers, so 4 on the real log and 4 on a scratch log can run at once |
| `meta worker start <lease>` | a worker gave its verdict and its session has not ended | it still counts towards the limit |
| `meta worker start <lease>` | the log is real and the build is not from main | refused, as for any write; nothing starts |
| `meta worker start <lease>` | the log is real | `self/workers.toml` is read from main; the working tree's copy is ignored |
| `meta worker start <lease>` | the log is scratch | any build runs, with the working tree's `self/workers.toml`, so a branch can try the file it changes |
| `meta worker start <lease>` | `self/workers.toml` cannot be read, or an entry has no source | error; nothing starts; the lease stays unused |
| `meta worker start <lease>` | `claude --version` is below the floor in `self/workers.toml` | refused naming both; nothing starts; the lease stays unused |
| `meta worker start <lease>` | `claude --version` fails, or its reply does not parse | error; nothing starts; the lease stays unused |
| `meta worker start <lease>` | the PR is no longer at the lease's head | refused naming both heads, with `meta review` as the next step; nothing starts |
| `meta worker start <lease>` | GitHub gives no view or diff | error; nothing starts; the lease stays unused |
| `meta worker start <lease>` | the session cannot be started | error; nothing is recorded; the lease stays unused |
| `meta worker start <lease>` | the supervisor cannot be started | the session's group is ended; error; nothing is recorded; the lease stays unused and a retry works |
| `meta worker wait <lease>` | the session exited 0 and its last `result` event is a success with a valid structured verdict, and the PR is at the lease's head | events `Reviewed` on this lease at that head, then `Finished` with `end: { exited: 0 }`; the findings go to `supervisor.log` |
| `meta worker wait <lease>` | no valid structured verdict, a failed result, no result, or an unreadable transcript | event `Finished` only. No verdict is recorded, and `meta status` shows "review finished with no verdict" with the transcript; `supervisor.log` says why |
| `meta worker wait <lease>` | the PR left the lease's head, or GitHub stays unreachable for 60 s | the same: a verdict lands only on the head it judged |
| `meta worker wait <lease>` | the session exited with another status | event `Finished` with `end: { exited: <status> }`; no verdict, whatever the transcript says |
| `meta worker wait <lease>` | the session is gone and left no exit status | event `Finished` with `end: vanished`; no verdict |
| `meta worker wait <lease>` | the deadline passes while the session runs | the process group is killed by the port's group kill (SIGTERM, a short grace, SIGKILL); event `Finished` with `end: deadline`; no verdict |
| `meta worker wait <lease>` | the task is waiting on the owner | the deadline holds all the same: a running session never goes without one |
| `meta worker wait <lease>` | the kill or the process table read fails | it keeps watching and tries again |
| `meta worker wait <lease>` | the log cannot be read after the first look | it keeps the last look, so the deadline still holds |
| `meta worker wait <lease>` | the worker's end is already recorded | exits with no event |
| `meta worker wait <lease>` | the lease started no worker | refused, which ends the session |
| `meta worker wait <lease>` | the first look at the log fails | error, which ends the session |
| `meta stop <lease>` | a worker is running on the lease | its process group is killed; event `Finished` with `end: stopped` |
| `meta stop` | any workers are running | every group is killed and each is recorded; with none running, says so |
| `meta stop <lease>` | the lease started no worker, or its worker has ended | refused; nothing is killed |
| `meta stop` | the build may not write the log | it kills all the same, and says nothing was recorded: the kill switch always kills. The supervisor then records `vanished` if it can |
| `meta stop` | a group cannot be killed | error naming it; the others are still killed and recorded |
| `meta stop [<lease>]` | the log does not parse or cannot be opened | the live sessions are found from the `run.json` files, the leader check still guards each kill, and all (or the lease's) are killed; error saying which were killed and that nothing was recorded |
| `meta status` | a worker's end is not recorded | "running agents" lists `<lease>: agent worker-<n>, <task>, role review, <deadline>, transcript <path>`, even if its task has ended, since the process is still there |
| `meta status` | a worker's end is recorded | not listed |
| `meta status` | a worker is past its deadline and not yet killed | "overdue leases" lists it with its transcript |
| `meta hook pre-tool-use` with tool `SendMessage` | the canned `continue` names a worker | denied: a headless session is not a subagent, and the worker's lease stays finished |

`meta stop` kills inside the log's lock, so its supervisor, waiting on that lock, finds the end recorded and does not call the session vanished.

## What this piece leaves open

- **Live canary: pending, run by the lead after merge.** No session of this command has run with the `--restricted` flags in meta: any real session needs a real lease, so the lead runs the first one on a real lease once this is on main. It should also try a read outside the scratch directory.
- **A worker with no supervisor stays listed.** If the machine restarts, the sessions are gone but the log still shows them running, and they count towards the limit. `meta stop` clears them: the port sees that the leader is not theirs, kills nothing, and records the end.
- **A lost `Finished`.** If the log cannot be written when a worker ends, the supervisor reports it in `supervisor.log` and exits. `meta stop` clears that too.
