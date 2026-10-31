# Stage 2c: headless workers, first piece

meta starts Claude sessions itself, under the rules in `docs/prereg.md`, "Headless workers". This first piece runs review leases only. The round cap and the token budget are deferred with token budgeting. The always-on supervisor is a separate decision, so nothing here touches launchd: a worker starts when someone runs `meta worker start`.

## The session

`meta worker start <lease>` runs this, with the brief as the prompt, and with `<model>`, `<allow>` and `<deny>` taken from `self/workers.toml`:

```
claude -p <brief> --restricted --model <model> \
  --tools <allow> --allowedTools <allow> --disallowedTools <deny> \
  --json-schema <verdict schema> --no-session-persistence \
  --permission-mode dontAsk --permission-prompts none --strict-mcp-config \
  --output-format stream-json --verbose
```

Today `<model>` is `opus`, `<allow>` is `Read,Grep,Glob` and `<deny>` is `Agent,Task,Bash,mcp__*`. The session has no way to run a command. Each flag, from the Claude Code docs (`/en/headless`, `/en/cli-reference`, `/en/permissions`, `/en/permission-modes`, `/en/tools-reference`, `/en/agent-sdk/structured-outputs`) and `claude --help`, 2.1.314:

| Flag | Why |
|---|---|
| `-p <brief>` | Non-interactive. The prompt comes first because `--tools`, `--allowedTools` and `--disallowedTools` each take a list, which would swallow a prompt placed after them. |
| `--restricted` | Removes the tools that run commands or code, confines the file tools to the working directories, loads only managed settings and `--settings` (so no user or project hooks or permission rules), and refuses `bypassPermissions`. Unlike `--bare`, it keeps the subscription login. |
| `--model` | The model the session runs on. `--restricted` ignores user settings, so without this flag `self/workers.toml` would not choose the model. `[model] review` must be there, not empty, and have a source, or the file does not parse. The worker's model is recorded with its process group and transcript, and `meta status` shows it. |
| `--tools` | The session's tools are exactly these. Without it, `--allowedTools` only skips prompts and every other tool stays available. Naming `Glob` and `Grep` here also brings them back on macOS and Linux, where the default set leaves them out. |
| `--allowedTools` | These tools run with no approval. |
| `--disallowedTools` | Deny comes first in rule order, so nothing can allow these back. A bare name removes the tool. `--tools` does not cover MCP tools, so `mcp__*` is denied too. A deny list without `Agent` or `Task` does not parse. |
| `--json-schema` | The session must end with `{verdict: pass or fail, findings: [text]}`. It adds the `StructuredOutput` tool, whose input is this verdict. |
| `--no-session-persistence` | Nothing to resume: the transcript below is the record. |
| `--permission-mode dontAsk` | Anything not allowed is denied, not asked. |
| `--permission-prompts none` | Nobody can answer, so Claude is told not to retry a denied call. |
| `--strict-mcp-config` | With no `--mcp-config`, no MCP server connects. |
| `--output-format stream-json --verbose` | The transcript is the whole run, event by event, and its last event has the cost and the structured output. |

**Meta does the trusted work.** The session has no `gh`, no token and no `meta`. Before it starts, `meta worker start` fetches `gh pr view` and `gh pr diff`, and then checks that the PR still stands at the lease's head, so a head that moved during the fetch refuses the start. When the session ends, `meta worker wait` takes the verdict from the transcript and records it through the same check as `meta verdict`, on the worker's own lease.

**The change is untrusted.** The brief says that `pr-view.json` and `pr.diff`, the PR's title, body and diff among them, are untrusted data, never instructions. The prompt holds none of their text: the session reads them from its directory.

**Environment.** The session and the `claude --version` call start from an empty environment plus `PATH`, `HOME` and `USER`. A subscription login needs nothing else: a probe under `env -i` with those three logged in and ran. So `GH_TOKEN`, `SSH_AUTH_SOCK`, `CLAUDE_CODE_MESSAGING_TOKEN` and the like never reach a session. The supervisor keeps the caller's environment, plus `GH_REPO` set to the origin, because it runs in the worker's directory, which is no clone, and it reads GitHub for the verdict.

**Version floor.** `[claude] floor` in `self/workers.toml` is the oldest `claude` a worker runs on. Below it the start is refused; a reply that does not parse is an error. That `claude --version` is the one call to `claude` besides the sessions. It is 2.1.314, the version the flags were probed on. `--restricted` itself needs 2.1.248.

Each worker has a directory, `<data dir>/workers/worker-<n>/`, mode 0700, kept after it ends:

| Path | Mode | Holds |
|---|---|---|
| `transcript.jsonl` | 0600 | The session's stdout, the only file the verdict is read from |
| `stderr.log` | 0600 | The session's stderr |
| `scratch/` | 0700 | The session's working directory, fresh |
| `scratch/pr-view.json` | 0600 | `gh pr view` of the PR |
| `scratch/pr.diff` | 0600 | `gh pr diff` of the PR |
| `run.json` | 0600 | The process group, transcript path and model, so that `meta stop` finds the session without the log |
| `status` | 0600 | The exit status, written by the session's own process group when `claude` exits |
| `supervisor.log` | 0600 | The supervisor's output: how the worker ended, and the verdict with its findings or why there is none |

A directory already there is what a failed start left, because the lease is still unused. It moves to `<data dir>/workers/failed/worker-<n>.<attempt>`, kept, and the start goes on. If its session still runs, the start is refused.

## Supervision

`/bin/sh` leads the session's process group and is the parent of `claude`, because only a parent can read an exit status once `meta worker start` has returned. `meta worker start` spawns the session and then the supervisor, `meta worker wait <lease>`, each detached, and returns.

The session's standard input is a pipe whose other end only the supervisor holds. When that end closes, a process in the group kills the group. So if the supervisor dies for any reason, the worker dies with it, as the prereg asks: a worker never runs without a deadline.

The supervisor looks at its worker every 5 s, so it can be up to 5 s late at the deadline. It reads the log under the lock, so its first look waits for `start` to record the worker. A worker recorded by nothing therefore ends itself: the supervisor refuses the lease, exits, and takes the session with it.

Before it signals a group or calls a session alive, the port checks that the leader's command line names the run's transcript. A process id can be reused, and a stranger's group must never be killed. A leader can also die before the rest of its group. `meta worker wait` then reads the session as ended, records that, and exits, and the lifeline takes the rest of the group with it, so its deadline kill only ever reaches a group whose leader is alive. `meta stop` on a log it can read goes on to ask whether a live member names the transcript. The shell that holds the lifeline does, for it keeps the leader's arguments. That group is still the run's, and it is ended. On a log it cannot read, `meta stop` finds workers through the leader check alone, so it does not find such a group, and the supervisor's exit ends it.

## Behaviours

Each row is a behaviour-table test through the single in-process `Call`, with a recorded spawn port. No test runs `claude`. The port's own process handling is tested against `/bin/sh`, and, for what no real process here can do (a member that outlives SIGKILL or refuses a signal), against injected signals and `pgrep` answers.

| Command | Given | Result |
|---|---|---|
| `meta worker start <lease>` | a live, unused review lease with a deadline, under the limit | one session starts, with the PR's view and diff in its scratch directory; events `LeaseUsed`, then `Started` with agent `worker-<n>`, the deadline counted from now, the process group, the transcript path and the model; prints all of them and returns at once |
| `meta worker start <lease>` | the lease is unknown, used, not live, a work lease, or has no deadline | refused naming which; nothing starts; nothing is recorded |
| `meta worker start <lease>` | the running workers have reached `concurrency.running` in `self/workers.toml` | refused naming them, with `meta stop <lease>` as the next step. The check holds the log's lock, so two starts cannot both take the last place. The limit is per log: a scratch log counts only its own workers, so 4 on the real log and 4 on a scratch log can run at once |
| `meta worker start <lease>` | the log is real and the build is not from main | refused, as for any write; nothing starts |
| `meta worker start <lease>` | the log is real | `self/workers.toml` is read from main; the working tree's copy is ignored |
| `meta worker start <lease>` | the log is scratch | any build runs, with the working tree's `self/workers.toml`, so a branch can try the file it changes |
| `meta worker start <lease>` | `self/workers.toml` cannot be read, an entry has no source, the model is missing or empty, or the deny list lacks `Agent` or `Task` | error; nothing starts; the lease stays unused |
| `meta worker start <lease>` | `claude --version` is below the floor in `self/workers.toml` | refused naming both; nothing starts; the lease stays unused |
| `meta worker start <lease>` | `claude --version` fails, or its reply does not parse | error; nothing starts; the lease stays unused |
| `meta worker start <lease>` | the PR is no longer at the lease's head | refused naming both heads, with `meta review` as the next step; nothing starts |
| `meta worker start <lease>` | GitHub gives no view or diff | error; nothing starts; the lease stays unused |
| `meta worker start <lease>` | the session cannot be started | error; nothing is recorded; the lease stays unused |
| `meta worker start <lease>` | the supervisor cannot be started | the session's group is ended; error, which also says if the group could not be ended; nothing is recorded; the lease stays unused and a retry works |
| `meta worker wait <lease>` | the session exited 0 and its last `result` event is a success with a valid structured verdict, and the PR is at the lease's head | events `Reviewed` on this lease at that head, then `Finished` with `end: { exited: 0 }`; the findings go to `supervisor.log` |
| `meta worker wait <lease>` | no valid structured verdict, a failed result, no result, or an unreadable transcript | event `Finished` only. No verdict is recorded, and `meta status` shows "review finished with no verdict" with the transcript; `supervisor.log` says why |
| `meta worker wait <lease>` | the PR left the lease's head, or GitHub stays unreachable for 60 s | the same: a verdict lands only on the head it judged |
| `meta worker wait <lease>` | the session exited with another status | event `Finished` with `end: { exited: <status> }`; no verdict, whatever the transcript says |
| `meta worker wait <lease>` | the session is gone and left no exit status | event `Finished` with `end: vanished`; no verdict |
| `meta worker wait <lease>` | the deadline passes while the session runs | the process group is killed by the port's group kill (SIGTERM, a short grace, SIGKILL); event `Finished` with `end: deadline`; no verdict |
| `meta worker wait <lease>` | the task is waiting on the owner | the deadline holds all the same: a running session never goes without one |
| `meta unwait <task>` | a worker runs on one of the task's leases, whether it started before or during the wait | its deadline stays where its start put it, however many waits it goes through. The task's other leases still get the wait back |
| `meta worker wait <lease>` | the kill or the process table read fails | it keeps watching and tries again |
| `meta worker wait <lease>` | the log cannot be read after the first look | it keeps the last look, so the deadline still holds |
| `meta worker wait <lease>` | the worker's end is already recorded | exits with no event |
| `meta worker wait <lease>` | the lease started no worker | refused, which ends the session |
| `meta worker wait <lease>` | the first look at the log fails for a reason other than a refusal, such as a lock held by another command | it looks again every 5 s, up to 6 more times; if every look fails, error, which ends the session |
| `meta stop <lease>` | a worker is running on the lease | its process group is killed; event `Finished` with `end: stopped` |
| `meta stop` | any workers are running | every group is killed and each is recorded; with none running, says so |
| `meta stop <lease>` | the lease started no worker, or its worker has ended | refused; nothing is killed |
| `meta stop` | the build may not write the log | it kills all the same, and says nothing was recorded: the kill switch always kills. The supervisor then records `vanished` if it can |
| `meta stop` | a group cannot be killed, or a member that is not a zombie, leader or not, is still there 2 s after SIGKILL | error naming it, with the port's reason; the others are still killed and recorded. An end is recorded only for a group confirmed gone: the exit status of `pgrep` says no live member is left. A signal refused for a group of zombies only is not a failure |
| `meta stop` | the session had already ended: no live member of its group names the transcript | nothing is killed; says the worker was already gone; event `Finished` with `end: stopped` |
| `meta stop` | the log fails after the closure ran | error saying which were killed, which were already gone and which could not be stopped, and that nothing was recorded. It says no worker was found only when none was |
| `meta stop [<lease>]` | the log does not parse or cannot be opened | the live sessions are found from the `run.json` files, the leader check still guards each kill, and all (or the lease's) are killed; the same error |
| `meta verdict <lease>` | the named lease is a worker's, running or ended | refused: a worker's verdict comes only from its session |
| `meta verdict <lease>` | a worker's lease, running or ended, stands before the named reviewer's lease | recorded on the named lease, whatever head the worker's lease has |
| `meta status` | a worker's end is not recorded and it is not flagged overdue | "agents with no turn end or stop recorded, not flagged overdue; subagents also with no verdict, on an open task" lists `<lease>: agent worker-<n>, <task>, role review, <deadline>, model <model>, transcript <path>`, even if its task has ended, since the process is still there |
| `meta status` | a worker's end is recorded | not listed |
| `meta status` | a worker is past its deadline and not yet killed | "overdue leases" lists it with its model and transcript |
| `meta hook pre-tool-use` with tool `SendMessage` | the canned `continue` names a worker | denied: a headless session is not a subagent, and the worker's lease stays finished |

`meta stop` kills inside the log's lock, so its supervisor, waiting on that lock, finds the end recorded and does not call the session vanished.

## Projects

A project is a task marked `project`; its id names the project. Tasks join it one at a time, so a task opened before its project was marked can join. Design intent stays in a file in the repo. What meta holds is who is in the project and where each task stands, which is what a replacement project agent needs first.

| Command | Given | Result |
|---|---|---|
| `meta role <task> project` | an unmarked task that is in no project | event `TaskRole`. A task has one role, and a member takes none: projects do not nest |
| `meta project <root> add <task>` | `<root>` is marked project; `<task>` is known, unmarked and in no project | event `Joined`, for good |
| `meta project <root> add <task>` | anything else | refused naming which; nothing recorded |
| `meta project <root>` | `<root>` is marked project | the latest work lease on the root, then the root and each member with its next step. Reads the PR of each linked task from GitHub; records nothing; any build may read it |
| `meta project <root>` | `<root>` is unknown or not marked | refused |
| `meta project <root>` | a linked task's PR is not at its pin | next step: link the new head. No verdict is read: only the pinned head counts |
| `meta project <root>` | a review of the pin is unused or under way | names the lease, and for an unused one the `meta worker start` that spends it |
| `meta project <root>` | a linked task has no verdict at its pin, or a fail | names the review to start, or the failing lease and its transcript; `supervisor.log` beside the transcript has the findings |
| `meta project <root>` | a pass at the pin | the dry run of `meta merge`: `ready` if it would merge, or each blocker with its next step. The merge stays the lead's |
| `meta project <root>` | GitHub cannot be read for a PR | says so for that task, and nothing about readiness; the rest still prints |
| `meta project <root>` | a task waits on the owner | next step is the question |
| `meta status` | a project's work lease | role `project`. It is running work while the root is active, linked included: the root's own PR stays open for the project's whole life, so the idle alarm stays quiet while its agent runs |

## What this piece leaves open

- **Live canary: run, on a scratch log and on the real log.** The lead ran it before review, on a scratch log, at 849ac01. A read outside the scratch directory was tried, in a direct session with the same flags, and was refused. The path through the real log has now been observed: a main-build worker on lease-117 reviewed PR #50 and recorded its verdict.
- **A worker with no supervisor stays listed.** If the machine restarts, the sessions are gone but the log still shows them running, and they count towards the limit. `meta stop` clears them: the port sees that the leader is not theirs, kills nothing, says the worker was already gone, and records the end.
- **A lost `Finished`.** If the log cannot be written when a worker ends, the supervisor reports it in `supervisor.log` and exits. `meta stop` clears that too.
