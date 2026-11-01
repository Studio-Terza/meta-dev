"""Which tests kill which mutants, read from cargo-mutants per-mutant logs.

shard   <mutants.out> <out.json>     one shard: its baseline tests and the tests that failed per mutant
report  <shards-dir> <sources> <report.md> <report.json>
                                     all shards: survivors, tests that kill nothing,
                                     mutants killed with no golden test failing
"""

import json
import pathlib
import re
import sys

RUNNING = re.compile(r"^\s*Running .*\(target/[^/]+/deps/([^)\s]+)\)")
DOCTESTS = re.compile(r"^\s*Doc-tests (\S+)")
RESULT = re.compile(r"^test (.+) \.\.\. (ok|FAILED)\s*$")
TEST_ATTR = re.compile(r"#\[test\]|#\[tokio::test\]")
FN_NAME = re.compile(r"fn\s+(\w+)")
GOLDEN_USE = re.compile(r"golden!\s*\(|Golden::|\.golden\(")
LIMIT = 300


def results(log_text):
    binary = ""
    ran = {}
    for line in log_text.splitlines():
        found = RUNNING.match(line)
        if found:
            binary = found.group(1)
            continue
        found = DOCTESTS.match(line)
        if found:
            binary = "doc-" + found.group(1)
            continue
        found = RESULT.match(line)
        if found:
            ran[binary + " :: " + found.group(1)] = found.group(2) == "ok"
    return ran


def log_of(outcome, out_dir):
    paths = [outcome.get("log_path")]
    paths += [phase.get("log_path") for phase in outcome.get("phase_results", [])]
    for path in paths:
        if path and (out_dir / path).is_file():
            return (out_dir / path).read_text(errors="replace")
    return ""


def mutant_name(scenario):
    mutant = scenario["Mutant"]
    if mutant.get("name"):
        return mutant["name"]
    return "{}: {}".format(mutant.get("file", "?"), mutant.get("replacement", ""))


def shard(out_dir, destination):
    out_dir = pathlib.Path(out_dir)
    outcomes = json.loads((out_dir / "outcomes.json").read_text())["outcomes"]
    baseline = []
    mutants = []
    for outcome in outcomes:
        scenario = outcome["scenario"]
        ran = results(log_of(outcome, out_dir))
        if scenario == "Baseline":
            baseline = sorted(ran)
        else:
            failed = sorted(name for name, passed in ran.items() if not passed)
            mutants.append({"name": mutant_name(scenario), "summary": outcome["summary"], "failed": failed})
    pathlib.Path(destination).write_text(json.dumps({"baseline": baseline, "mutants": mutants}))


def golden_tests(sources):
    names = set()
    for path in pathlib.Path(sources).glob("crates/*/**/*.rs"):
        chunks = TEST_ATTR.split(path.read_text(errors="replace"))[1:]
        for chunk in chunks:
            named = FN_NAME.search(chunk)
            if named and GOLDEN_USE.search(chunk.split("\n#[", 1)[0]):
                names.add(named.group(1))
    return names


def is_golden(test, golden):
    return test.rsplit("::", 1)[-1].strip() in golden


def bullets(items):
    lines = ["- " + item for item in items[:LIMIT]]
    if len(items) > LIMIT:
        lines.append("- ... {} more in report.json".format(len(items) - LIMIT))
    return lines or ["none"]


def report(shards_dir, sources, markdown, destination):
    baseline = set()
    mutants = []
    for path in sorted(pathlib.Path(shards_dir).glob("*/matrix.json")):
        data = json.loads(path.read_text())
        baseline.update(data["baseline"])
        mutants.extend(data["mutants"])
    golden = golden_tests(sources)
    killers = set()
    for mutant in mutants:
        killers.update(mutant["failed"])
    surviving = sorted(m["name"] for m in mutants if m["summary"] == "MissedMutant")
    timed_out = sorted(m["name"] for m in mutants if m["summary"] == "Timeout")
    idle = sorted(baseline - killers)
    ungolden = [
        "{} -> {}".format(m["name"], ", ".join(m["failed"]))
        for m in sorted(mutants, key=lambda m: m["name"])
        if m["failed"] and not any(is_golden(test, golden) for test in m["failed"])
    ]
    summary = {
        "mutants": len(mutants),
        "tests": len(baseline),
        "golden_tests": len(golden),
        "surviving": surviving,
        "timed_out": timed_out,
        "tests_that_kill_nothing": idle,
        "killed_with_no_golden_failing": ungolden,
    }
    destination_text = json.dumps(summary, indent=1)
    pathlib.Path(destination).write_text(destination_text)
    text = ["## Mutation matrix", "",
            "{} mutants, {} tests, {} golden tests.".format(len(mutants), len(baseline), len(golden)), "",
            "### Surviving mutants ({})".format(len(surviving)), ""] + bullets(surviving) + [
            "", "### Timed out mutants ({})".format(len(timed_out)), ""] + bullets(timed_out) + [
            "", "### Tests that kill no mutant ({})".format(len(idle)), ""] + bullets(idle) + [
            "", "### Mutants killed while no golden test fails ({})".format(len(ungolden)), ""] + bullets(ungolden)
    pathlib.Path(markdown).write_text("\n".join(text) + "\n")


if __name__ == "__main__":
    command, *arguments = sys.argv[1:]
    {"shard": shard, "report": report}[command](*arguments)
