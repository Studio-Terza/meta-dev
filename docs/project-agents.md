# Project agents: state of task-63

The first project agent's own memory (agent `project-agents`). A replacement continues from this file and `meta status`.

## Goal

Meta supports project agents as `docs/prereg.md` describes them ("Agents"), and this project is the proof: first by instruction, then with those instructions replaced by what meta does. Keep it small. Add to meta only what this project has shown it needs.

## Design

| Piece | What | Why it is needed |
|---|---|---|
| Project | A task with role `project`. Its id is the project's id. | Prereg: one project agent per project, replaced from its state in meta. Today meta has no idea which tasks belong together. |
| Member | `meta project <root> add <task>`: one log event per joined task. | Tasks opened before the event existed must be able to join, so it is not a field of `Opened`. |
| View | `meta project <root>`: each member's stage, its verdict at the pinned head and its next step. For a pass, a dry run of `meta merge`. | Replaces this file's task table, "record it in your state file", and the lead's reading of PRs. |
| Brief | `meta claim <root> --agent <name>` prints the standing order. A fresh lease on the root is the replacement. | Replaces the task-63 prompt. |

Left out until a need shows: headless project agents (prereg rollout: after data from fixers and reviewers), implementer workers, round caps.

Design intent stays in a file in the repo, because meta stores only titles (task-59).

## Tasks

| Task | PR | What | State |
|---|---|---|---|
| task-63 | see below | Root. Carries this file. | open |
| task-64 | | Project view | next |
| task-65 | | Project agent brief. Comes after task-64, because it reads the role. | not started |
| task-66 | | Project branches | waits on the owner: gates on project branches |

## How to continue

1. `meta status`, then this file.
2. Take the first task that is not done or waiting. Branch off main, one PR per task, no stacks.
3. Push, `meta link` from the clone, watch CI, `meta review <task> --agent <name>`, `meta worker start <lease>`.
4. Read the worker's findings from `supervisor.log` beside its transcript. Judge them against the code.
5. Pass at the current head with green CI on current main: note it here. The lead merges.

## Log of what being told cost

Gaps seen while working by instruction. Each is a candidate for meta, added only when real.

- `meta claim <task> --self` says "claimed by the lead" for a task a project agent holds.
