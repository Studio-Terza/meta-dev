use super::command::{end_group, run_found, run_prepared};
use super::jsonl;
use super::log::home;
use crate::{Listed, PortError, Sessions};
use meta_core::{
    AgentId, Binary, ExitStatus, Model, Origin, ProcessGroup, ReviewCopy, Session, Stopped, WorkerPlan, WorkerRun,
};
use std::ffi::OsString;
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{ErrorKind, PipeReader, PipeWriter, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Runs the command given after the status, transcript and stderr paths, as the process group's
/// leader, and stays its parent, for only a parent can read the exit status. Standard input is a
/// pipe whose other end only the supervisor holds: when it closes, the supervisor is gone, so the
/// group goes too. The script ends by killing the group, so nothing a session left behind outlives
/// it. Stderr has its own file: the verdict is read from the transcript, stdout alone.
const SESSION: &str = r#"umask 077
status=$1; transcript=$2; errors=$3; shift 3
exec 3<&0
{ cat <&3 >/dev/null; kill -KILL 0; } &
"$@" </dev/null 3<&- >"$transcript" 2>"$errors" &
wait $!
printf %s "$?" >"$status.part" && mv "$status.part" "$status"
kill -KILL 0
"#;

/// Under the data directory, a directory per worker, kept after it ends.
const WORKERS: &str = "workers";
/// Beside the worker directories: those of starts that failed, kept and out of the way of a retry.
const FAILED: &str = "failed";
const TRANSCRIPT: &str = "transcript.jsonl";
const STDERR: &str = "stderr.log";
const STATUS: &str = "status";
const SUPERVISOR: &str = "supervisor.log";
const SCRATCH: &str = "scratch";
/// The session's process group and transcript, so that `meta stop` finds it without the log.
const RUN: &str = "run.json";

/// All a session's environment. The subscription login needs nothing more: `claude` finds it from
/// `HOME` (probed under `env -i`). Every token and socket of the caller stays behind.
const PASSED: [&str; 3] = ["PATH", "HOME", "USER"];

/// Private to the user, whatever the umask: a transcript holds the session's whole run.
const PRIVATE_DIR: u32 = 0o700;
const PRIVATE_FILE: u32 = 0o600;

#[derive(Debug, Clone, Copy)]
pub struct HeadlessSessions;

/// What a session runs and reads.
struct Launch<'a> {
    program: &'a str,
    args: &'a [String],
    copy: &'a ReviewCopy,
    model: &'a Model,
}

/// `error` as a port error that names `path`.
fn at(path: &Path, error: std::io::Error) -> PortError {
    PortError::new(format!("{}: {error}", path.display()))
}

fn worker_root() -> Result<PathBuf, PortError> {
    home().map(|home| home.join(WORKERS))
}

fn worker_dir(worker: &AgentId) -> Result<PathBuf, PortError> {
    worker_root().map(|root| root.join(worker.to_string()))
}

/// The variables of `all` that a session gets.
fn passed(all: impl Iterator<Item = (OsString, OsString)>) -> Vec<(OsString, OsString)> {
    all.filter(|(name, _)| PASSED.iter().any(|kept| name == kept)).collect()
}

/// Narrows `command` to a session's environment.
fn narrowed(command: &mut Command) {
    command.env_clear().envs(passed(std::env::vars_os()));
}

fn private_file(path: &Path) -> Result<File, PortError> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true).mode(PRIVATE_FILE).open(path).map_err(|error| at(path, error))
}

fn write_private(path: &Path, text: &str) -> Result<(), PortError> {
    private_file(path)?.write_all(text.as_bytes()).map_err(|error| at(path, error))
}

/// The run recorded in `dir`, if one is.
fn recorded(dir: &Path) -> Result<Option<WorkerRun>, PortError> {
    let path = dir.join(RUN);
    match fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|error| PortError::new(format!("{}: {error}", path.display()))),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(at(&path, error)),
    }
}

fn record(dir: &Path, run: &WorkerRun) -> Result<(), PortError> {
    let text = serde_json::to_string(run).map_err(|error| PortError::new(error.to_string()))?;
    let (part, path) = (dir.join(format!("{RUN}.part")), dir.join(RUN));
    write_private(&part, &text)?;
    fs::rename(&part, &path).map_err(|error| at(&path, error))
}

/// Makes way for a start in `dir`. The ledger holds the lease unused, so anything there is what a
/// start that failed left: kept under `FAILED`, unless its session still runs, which refuses.
fn make_way(dir: &Path) -> Result<(), PortError> {
    if !dir.exists() {
        return Ok(());
    }
    if let Some(run) = recorded(dir)?
        && is_alive(&run)?
    {
        return Err(PortError::new(format!("{}: a session still runs here", dir.display())));
    }
    let failed = dir.with_file_name(FAILED);
    DirBuilder::new().recursive(true).mode(PRIVATE_DIR).create(&failed).map_err(|error| at(&failed, error))?;
    let name = dir.file_name().unwrap_or_default().to_string_lossy().into_owned();
    let free = (1u32..).map(|attempt| failed.join(format!("{name}.{attempt}"))).find(|path| !path.exists());
    let aside = free.ok_or_else(|| PortError::new(format!("{}: no free name", failed.display())))?;
    fs::rename(dir, &aside).map_err(|error| at(&aside, error))
}

/// Starts the session in a fresh, private directory under `dir`, in a process group of its own,
/// detached from the caller. It ends when the writer of `lifeline` closes.
fn launch(dir: &Path, session: &Launch<'_>, lifeline: PipeReader) -> Result<WorkerRun, PortError> {
    let scratch = dir.join(SCRATCH);
    let mut folders = DirBuilder::new();
    folders.recursive(true).mode(PRIVATE_DIR).create(dir).map_err(|error| at(dir, error))?;
    folders.recursive(false).create(&scratch).map_err(|error| at(&scratch, error))?;
    write_private(&scratch.join(ReviewCopy::VIEW), &session.copy.view)?;
    write_private(&scratch.join(ReviewCopy::DIFF), &session.copy.diff)?;
    let transcript = dir.join(TRANSCRIPT);
    private_file(&transcript)?;
    let errors = dir.join(STDERR);
    private_file(&errors)?;
    let mut command = Command::new("/bin/sh");
    narrowed(&mut command);
    let leader = command
        .args(["-c", SESSION, "meta-worker"])
        .arg(dir.join(STATUS))
        .arg(&transcript)
        .arg(&errors)
        .arg(session.program)
        .args(session.args)
        .current_dir(&scratch)
        .stdin(lifeline)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(|error| at(&scratch, error))?;
    // Detached: the script outlives this process, and nothing here waits for it.
    Ok(WorkerRun {
        group: ProcessGroup::of(leader.id()),
        transcript: transcript.display().to_string().parse().map_err(|error| PortError::new(format!("{error}")))?,
        model: session.model.clone(),
    })
}

/// Starts `meta worker wait`, or what stands for it, holding the session's lifeline. It keeps the
/// caller's environment, for it reads GitHub and the log; the session does not get these.
fn supervise(
    dir: &Path,
    binary: &Binary,
    args: &[String],
    repository: &Origin,
    lifeline: PipeWriter,
) -> Result<(), PortError> {
    let log = dir.join(SUPERVISOR);
    let output = private_file(&log)?;
    let errors = output.try_clone().map_err(|error| at(&log, error))?;
    Command::new(binary.to_string())
        .args(args)
        .current_dir(dir)
        .env("GH_REPO", repository.to_string())
        .stdin(lifeline)
        .stdout(output)
        .stderr(errors)
        .process_group(0)
        .spawn()
        .map(drop)
        .map_err(|error| at(dir, error))
}

/// Starts the session in `dir` and hands its lifeline to `join`. When that fails, the group is
/// ended here, so that the failed start leaves no session and the lease can be tried again.
fn start_in(
    dir: &Path,
    session: &Launch<'_>,
    join: impl FnOnce(PipeWriter) -> Result<(), PortError>,
) -> Result<WorkerRun, PortError> {
    make_way(dir)?;
    let (reader, writer) = std::io::pipe().map_err(|error| at(dir, error))?;
    let run = launch(dir, session, reader)?;
    let joined = record(dir, &run).and_then(|()| join(writer));
    joined.map_err(|error| abandoned(&error, end_group(run.group.leader())))?;
    Ok(run)
}

/// Why a start failed, and, if its session could not be ended, that too: that group runs on with
/// nothing recording it.
fn abandoned(error: &PortError, ended: Result<Stopped, PortError>) -> PortError {
    match ended {
        Ok(_) => error.clone(),
        Err(left) => PortError::new(format!("{error}; the session could not be ended either: {left}")),
    }
}

/// Whether the leader of the run's group is alive and is that run's: its command line names the
/// transcript. A process id can be reused, and a stranger's group must never be killed.
fn is_alive(run: &WorkerRun) -> Result<bool, PortError> {
    let leader = run.group.leader().to_string();
    let command = run_found("ps", &["-ww", "-o", "command=", "-p", &leader])?;
    Ok(command.is_some_and(|command| command.contains(&run.transcript.to_string())))
}

fn exit_status(dir: &Path) -> Option<ExitStatus> {
    fs::read_to_string(dir.join(STATUS)).ok()?.parse().ok().map(ExitStatus::of)
}

fn state_in(dir: &Path, run: &WorkerRun) -> Result<Session, PortError> {
    Ok(match is_alive(run)? {
        true => Session::Running,
        false => Session::Ended(exit_status(dir)),
    })
}

/// The worker in `dir`, when a session recorded there is alive.
fn alive_in(dir: &Path) -> Result<Option<(AgentId, WorkerRun)>, PortError> {
    let Some(run) = recorded(dir)? else { return Ok(None) };
    if !is_alive(&run)? {
        return Ok(None);
    }
    Ok(Some((dir.file_name().unwrap_or_default().to_string_lossy().parse()?, run)))
}

fn listed_in(root: &Path) -> Result<Listed, PortError> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(at(root, error)),
    };
    let found = entries.map(|entry| alive_in(&entry.map_err(|error| at(root, error))?.path()));
    Ok(found.filter_map(Result::transpose).collect())
}

impl Sessions for HeadlessSessions {
    fn version(&self) -> Result<String, PortError> {
        run_prepared("claude", &["--version"], narrowed)
    }

    fn start(&mut self, plan: &WorkerPlan) -> Result<WorkerRun, PortError> {
        let dir = worker_dir(plan.id())?;
        let args = plan.arguments();
        let session = Launch { program: plan.program(), args: &args, copy: plan.copy(), model: plan.model() };
        // The writer goes to the supervisor; dropped on the way, it ends the session too.
        start_in(&dir, &session, |writer| {
            supervise(&dir, plan.binary(), &plan.supervisor_arguments(), plan.repository(), writer)
        })
    }

    fn state(&self, worker: &AgentId, run: &WorkerRun) -> Result<Session, PortError> {
        state_in(&worker_dir(worker)?, run)
    }

    fn stop(&mut self, run: &WorkerRun) -> Result<Stopped, PortError> {
        match is_alive(run)? {
            true => end_group(run.group.leader()),
            false => Ok(Stopped::AlreadyGone),
        }
    }

    fn transcript(&self, run: &WorkerRun) -> Result<Vec<String>, PortError> {
        let path = PathBuf::from(run.transcript.to_string());
        fs::read_to_string(&path).map(|text| jsonl::lines(&text)).map_err(|error| at(&path, error))
    }

    fn listed(&self) -> Result<Listed, PortError> {
        listed_in(&worker_root()?)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HeadlessSessions, Launch, PASSED, RUN, STATUS, STDERR, SUPERVISOR, TRANSCRIPT, abandoned, is_alive, launch,
        listed_in, make_way, passed, recorded, start_in, state_in, supervise,
    };
    use crate::{PortError, Sessions};
    use meta_core::{ExitStatus, Model, ReviewCopy, Session, SessionVerdict, Stopped, Verdict, WorkerRun};
    use meta_testkit::Scratch;
    use std::ffi::OsString;
    use std::fs;
    use std::io::{PipeWriter, Write};
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::thread;
    use std::time::{Duration, Instant};

    const REPOSITORY: &str = "https://github.com/Studio-Terza/meta-dev.git";

    fn copy() -> ReviewCopy {
        ReviewCopy { view: "the view\n".to_owned(), diff: "the diff\n".to_owned() }
    }

    fn model() -> Model {
        "opus".parse().unwrap()
    }

    fn sh(script: &str) -> [String; 2] {
        ["-c".to_owned(), script.to_owned()]
    }

    /// A session of `/bin/sh -c <script>`; no real `claude` runs in tests.
    fn session(dir: &Path, script: &str) -> (WorkerRun, PipeWriter) {
        let (reader, writer) = std::io::pipe().unwrap();
        let args = sh(script);
        let launched = Launch { program: "/bin/sh", args: &args, copy: &copy(), model: &model() };
        (launch(dir, &launched, reader).unwrap(), writer)
    }

    fn ended(dir: &Path, run: &WorkerRun) -> Session {
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(10) {
            if let Ok(Session::Ended(status)) = state_in(dir, run) {
                return Session::Ended(status);
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!("the session never ended");
    }

    fn mode(path: &Path) -> u32 {
        fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn a_failed_start_says_so_when_its_session_could_not_be_ended_either() {
        let why = PortError::new("no supervisor");
        assert_eq!(abandoned(&why, Ok(Stopped::Killed)), why);
        assert_eq!(abandoned(&why, Ok(Stopped::AlreadyGone)), why);
        let left = abandoned(&why, Err(PortError::new("process group 7: still there after SIGKILL")));
        assert_eq!(
            left.to_string(),
            "no supervisor; the session could not be ended either: process group 7: still there after SIGKILL"
        );
    }

    #[test]
    fn a_session_keeps_its_output_and_exit_status_and_reads_the_copy_in_a_fresh_directory() {
        let root = Scratch::new("worker-exit");
        let dir = root.join("worker-1");
        let script = "pwd; cat pr-view.json pr.diff; printf oops >&2; exit 3";
        let (run, _lifeline) = session(&dir, script);
        assert_eq!(ended(&dir, &run), Session::Ended(Some(ExitStatus::of(3))));
        let written = fs::read_to_string(dir.join(TRANSCRIPT)).unwrap();
        assert!(written.contains("/worker-1/scratch\n") && written.contains("the view\nthe diff\n"), "{written}");
        assert!(!written.contains("oops"), "{written}");
        assert_eq!(fs::read_to_string(dir.join(STDERR)).unwrap(), "oops");
        assert_eq!(HeadlessSessions.transcript(&run).unwrap().last().unwrap(), "the diff");
    }

    /// A result event of the shape `SessionVerdict` reads.
    fn result(verdict: &str) -> String {
        format!(
            r#"{{"type":"result","subtype":"success","is_error":false,"structured_output":{{"verdict":"{verdict}","findings":[]}}}}"#
        )
    }

    #[test]
    fn a_result_on_stderr_neither_changes_the_verdict_nor_stands_for_one() {
        let root = Scratch::new("worker-stderr");
        let read = |name: &str, script: String| {
            let dir = root.join(name);
            let (run, _lifeline) = session(&dir, &script);
            ended(&dir, &run);
            SessionVerdict::from_transcript(&HeadlessSessions.transcript(&run).unwrap())
        };
        let (pass, fail) = (result("pass"), result("fail"));
        let judged = read("worker-1", format!("echo '{pass}'; echo '{fail}' >&2"));
        assert_eq!(judged.unwrap().verdict(), Verdict::Pass);
        let forged = read("worker-2", format!("echo '{pass}' >&2"));
        assert!(forged.unwrap_err().to_string().contains("holds no result"));
    }

    #[test]
    fn a_session_gets_only_the_passed_variables_and_no_token_or_socket_is_among_them() {
        let names = |pairs: Vec<(OsString, OsString)>| -> Vec<String> {
            pairs.into_iter().map(|(name, _)| name.to_string_lossy().into_owned()).collect()
        };
        let secrets = [
            "GH_TOKEN",
            "GITHUB_TOKEN",
            "SSH_AUTH_SOCK",
            "CLAUDE_CODE_MESSAGING_TOKEN",
            "ANTHROPIC_API_KEY",
            "XDG_DATA_HOME",
        ];
        let all = PASSED.iter().chain(&secrets).map(|name| (OsString::from(name), OsString::from("value")));
        assert_eq!(names(passed(all)), PASSED);
        let root = Scratch::new("worker-env");
        let dir = root.join("worker-1");
        let (run, _lifeline) = session(&dir, "env");
        ended(&dir, &run);
        let shell = ["PWD", "OLDPWD", "SHLVL", "_", "__CF_USER_TEXT_ENCODING"];
        let written = HeadlessSessions.transcript(&run).unwrap();
        let foreign: Vec<&String> = written
            .iter()
            .filter(|line| {
                !PASSED
                    .iter()
                    .chain(&shell)
                    .any(|name| line.as_bytes().get(..=name.len()) == Some(format!("{name}=").as_bytes()))
            })
            .collect();
        assert_eq!(foreign, Vec::<&String>::new());
        assert!(
            std::env::vars_os().any(|(name, _)| name == "CARGO_PKG_NAME"),
            "cargo sets variables a session must not get"
        );
    }

    #[test]
    fn the_worker_directory_and_everything_a_session_writes_are_private_to_the_user() {
        let root = Scratch::new("worker-modes");
        let dir = root.join("worker-1");
        let (run, _lifeline) = session(&dir, "printf output; printf oops >&2; exit 0");
        ended(&dir, &run);
        assert_eq!((mode(&dir), mode(&dir.join("scratch"))), (0o700, 0o700));
        assert_eq!((mode(&dir.join(TRANSCRIPT)), mode(&dir.join(STDERR))), (0o600, 0o600));
        assert_eq!(mode(&dir.join(STATUS)), 0o600);
        assert_eq!(mode(&dir.join("scratch").join(ReviewCopy::DIFF)), 0o600);
    }

    #[test]
    fn a_directory_whose_session_still_runs_refuses_the_start_and_one_left_by_a_failed_start_is_kept_aside() {
        let root = Scratch::new("worker-fresh");
        let dir = root.join("worker-1");
        let (run, _lifeline) = session(&dir, "sleep 30");
        super::record(&dir, &run).unwrap();
        let refused = make_way(&dir).unwrap_err().to_string();
        assert!(refused.contains("a session still runs here"), "{refused}");
        HeadlessSessions.stop(&run).unwrap();
        ended(&dir, &run);
        make_way(&dir).unwrap();
        assert!(!dir.exists());
        assert!(root.join("failed/worker-1.1").join(TRANSCRIPT).exists());
        session(&dir, "exit 0");
        make_way(&dir).unwrap();
        assert!(root.join("failed/worker-1.2").join(TRANSCRIPT).exists());
    }

    #[test]
    fn a_supervisor_that_cannot_start_leaves_no_session_and_the_next_start_goes_through() {
        let root = Scratch::new("worker-retry");
        let dir = root.join("worker-1");
        let args = sh("sleep 30");
        let launched = Launch { program: "/bin/sh", args: &args, copy: &copy(), model: &model() };
        let missing = "/nonexistent/meta".parse().unwrap();
        let repository = REPOSITORY.parse().unwrap();
        let failed =
            start_in(&dir, &launched, |writer| supervise(&dir, &missing, &[], &repository, writer)).unwrap_err();
        assert!(failed.to_string().contains("worker-1"), "{failed}");
        let run = recorded(&dir).unwrap().unwrap();
        assert_eq!(is_alive(&run), Ok(false));
        let shell = "/bin/sh".parse().unwrap();
        let script = sh("printf %s \"$GH_REPO\"; exec sleep 5");
        let again = start_in(&dir, &launched, |writer| supervise(&dir, &shell, &script, &repository, writer)).unwrap();
        assert_eq!(state_in(&dir, &again), Ok(Session::Running));
        let supervisor = dir.join(SUPERVISOR);
        let started = Instant::now();
        while fs::read_to_string(&supervisor).unwrap().is_empty() && started.elapsed() < Duration::from_secs(10) {
            thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(fs::read_to_string(&supervisor).unwrap(), REPOSITORY);
        assert_eq!(mode(&supervisor), 0o600);
        HeadlessSessions.stop(&again).unwrap();
    }

    #[test]
    fn a_failure_to_record_the_run_ends_the_session_too() {
        let root = Scratch::new("worker-unjoined");
        let dir = root.join("worker-1");
        let args = sh("sleep 30");
        let launched = Launch { program: "/bin/sh", args: &args, copy: &copy(), model: &model() };
        let refused = start_in(&dir, &launched, |_| Err(PortError::new("no supervisor")));
        assert_eq!(refused, Err(PortError::new("no supervisor")));
        assert_eq!(is_alive(&recorded(&dir).unwrap().unwrap()), Ok(false));
    }

    #[test]
    fn the_list_finds_live_sessions_from_their_directories_and_reports_an_unreadable_one() {
        let root = Scratch::new("worker-listed");
        assert_eq!(listed_in(&root.join("none")), Ok(Vec::new()));
        let (live, dead, broken) = (root.join("worker-1"), root.join("worker-2"), root.join("worker-3"));
        let (run, _lifeline) = session(&live, "sleep 30");
        super::record(&live, &run).unwrap();
        let (gone, _other) = session(&dead, "exit 0");
        ended(&dead, &gone);
        super::record(&dead, &gone).unwrap();
        fs::create_dir(&broken).unwrap();
        fs::write(broken.join(RUN), "{").unwrap();
        fs::create_dir(root.join("failed")).unwrap();
        let listed = listed_in(root.as_ref()).unwrap();
        let (found, unreadable): (Vec<_>, Vec<_>) = listed.into_iter().partition(Result::is_ok);
        assert_eq!(found, vec![Ok(("worker-1".parse().unwrap(), run.clone()))]);
        assert_eq!(unreadable.len(), 1, "{unreadable:?}");
        HeadlessSessions.stop(&run).unwrap();
    }

    #[test]
    fn when_the_supervisors_end_of_the_lifeline_closes_the_whole_group_goes_with_no_exit_status() {
        let root = Scratch::new("worker-lifeline");
        let dir = root.join("worker-1");
        let (run, mut lifeline) = session(&dir, "sleep 30 & sleep 30");
        assert_eq!(state_in(&dir, &run), Ok(Session::Running));
        lifeline.write_all(b"ignored").unwrap();
        assert_eq!(state_in(&dir, &run), Ok(Session::Running));
        drop(lifeline);
        assert_eq!(ended(&dir, &run), Session::Ended(None));
    }

    #[test]
    fn stop_ends_the_group_and_is_no_error_for_a_session_already_gone() {
        let root = Scratch::new("worker-stop");
        let dir = root.join("worker-1");
        let (run, _lifeline) = session(&dir, "sleep 30");
        assert_eq!(HeadlessSessions.stop(&run), Ok(Stopped::Killed));
        assert_eq!(ended(&dir, &run), Session::Ended(None));
        assert_eq!(HeadlessSessions.stop(&run), Ok(Stopped::AlreadyGone));
    }

    #[test]
    fn a_process_that_merely_has_the_recorded_id_is_not_the_session() {
        let root = Scratch::new("worker-stranger");
        let dir = root.join("worker-1");
        let (run, _lifeline) = session(&dir, "sleep 30");
        let stranger = WorkerRun { transcript: "/elsewhere/transcript.jsonl".parse().unwrap(), ..run.clone() };
        assert_eq!(is_alive(&stranger), Ok(false));
        assert_eq!(HeadlessSessions.stop(&stranger), Ok(Stopped::AlreadyGone));
        assert_eq!(state_in(&dir, &run), Ok(Session::Running));
        assert!(!dir.join(STATUS).exists());
        HeadlessSessions.stop(&run).unwrap();
    }
}
