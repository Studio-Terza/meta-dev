use super::command::{end_group, run_found};
use super::log::home;
use crate::{PortError, Sessions};
use meta_core::{AgentId, ExitStatus, Origin, ProcessGroup, Session, WorkerPlan, WorkerRun};
use std::fs::{self, File};
use std::io::PipeReader;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Runs the command given after the status and transcript paths, as the process group's leader,
/// and stays its parent, for only a parent can read the exit status. Standard input is a pipe whose
/// other end only the supervisor holds: when it closes, the supervisor is gone, so the group goes
/// too. The script ends by killing the group, so nothing a session left behind outlives it.
const SESSION: &str = r#"status=$1; transcript=$2; shift 2
exec 3<&0
{ cat <&3 >/dev/null; kill -KILL 0; } &
"$@" </dev/null >"$transcript" 2>&1 &
wait $!
printf %s "$?" >"$status.part" && mv "$status.part" "$status"
kill -KILL 0
"#;

/// Under the data directory, a directory per worker, kept after it ends.
const WORKERS: &str = "workers";
const TRANSCRIPT: &str = "transcript.jsonl";
const STATUS: &str = "status";
const SUPERVISOR: &str = "supervisor.log";
const SCRATCH: &str = "scratch";

#[derive(Debug, Clone, Copy)]
pub struct HeadlessSessions;

/// `error` as a port error that names `path`.
fn at(path: &Path, error: std::io::Error) -> PortError {
    PortError::new(format!("{}: {error}", path.display()))
}

fn worker_dir(worker: &AgentId) -> Result<PathBuf, PortError> {
    home().map(|home| home.join(WORKERS).join(worker.to_string()))
}

/// Starts `program` in a fresh directory under `dir`, in a process group of its own, detached from
/// the caller. It ends when the writer of `lifeline` closes.
fn launch(
    dir: &Path,
    program: &str,
    args: &[String],
    repository: &Origin,
    lifeline: PipeReader,
) -> Result<WorkerRun, PortError> {
    let scratch = dir.join(SCRATCH);
    fs::create_dir_all(dir).map_err(|error| at(dir, error))?;
    fs::create_dir(&scratch).map_err(|error| at(&scratch, error))?;
    let transcript = dir.join(TRANSCRIPT);
    let session = Command::new("/bin/sh")
        .args(["-c", SESSION, "meta-worker"])
        .arg(dir.join(STATUS))
        .arg(&transcript)
        .arg(program)
        .args(args)
        .current_dir(&scratch)
        .env("GH_REPO", repository.to_string())
        .stdin(lifeline)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .map_err(|error| at(&scratch, error))?;
    let run = WorkerRun {
        group: ProcessGroup::of(session.id()),
        transcript: transcript.display().to_string().parse().map_err(|error| PortError::new(format!("{error}")))?,
    };
    // Detached: the script outlives this process, and nothing here waits for it.
    drop(session);
    Ok(run)
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

impl Sessions for HeadlessSessions {
    fn start(&mut self, plan: &WorkerPlan) -> Result<WorkerRun, PortError> {
        let dir = worker_dir(plan.id())?;
        let (reader, writer) = std::io::pipe().map_err(|error| at(&dir, error))?;
        let run = launch(&dir, plan.program(), &plan.arguments(), plan.repository(), reader)?;
        let log = dir.join(SUPERVISOR);
        let output = File::create(&log).map_err(|error| at(&log, error))?;
        let errors = output.try_clone().map_err(|error| at(&log, error))?;
        // A supervisor that fails to start drops the writer, which ends the session.
        let supervisor = Command::new(plan.binary().to_string())
            .args(plan.supervisor_arguments())
            .current_dir(&dir)
            .stdin(writer)
            .stdout(output)
            .stderr(errors)
            .process_group(0)
            .spawn()
            .map_err(|error| at(&dir, error))?;
        drop(supervisor);
        Ok(run)
    }

    fn state(&self, worker: &AgentId, run: &WorkerRun) -> Result<Session, PortError> {
        state_in(&worker_dir(worker)?, run)
    }

    fn stop(&mut self, run: &WorkerRun) -> Result<(), PortError> {
        if is_alive(run)? {
            end_group(run.group.leader());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{HeadlessSessions, STATUS, TRANSCRIPT, is_alive, launch, state_in};
    use crate::Sessions;
    use meta_core::{ExitStatus, Session, WorkerRun};
    use meta_testkit::Scratch;
    use std::fs;
    use std::io::{PipeWriter, Write};
    use std::path::Path;
    use std::thread;
    use std::time::{Duration, Instant};

    const REPOSITORY: &str = "https://github.com/Studio-Terza/meta-dev.git";

    /// A session of `/bin/sh -c <script>`; no real `claude` runs in tests.
    fn session(dir: &Path, script: &str) -> (WorkerRun, PipeWriter) {
        let (reader, writer) = std::io::pipe().unwrap();
        let args = ["-c".to_owned(), script.to_owned()];
        (launch(dir, "/bin/sh", &args, &REPOSITORY.parse().unwrap(), reader).unwrap(), writer)
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

    #[test]
    fn a_session_keeps_its_output_and_exit_status_and_runs_in_a_fresh_directory_with_gh_repo_set() {
        let root = Scratch::new("worker-exit");
        let dir = root.join("worker-1");
        let (run, _lifeline) = session(&dir, r#"pwd; printf '%s\n' "$GH_REPO"; printf oops >&2; exit 3"#);
        assert_eq!(ended(&dir, &run), Session::Ended(Some(ExitStatus::of(3))));
        let written = fs::read_to_string(dir.join(TRANSCRIPT)).unwrap();
        assert!(written.contains("/worker-1/scratch\n") && written.contains(REPOSITORY), "{written}");
        assert!(written.contains("oops"), "{written}");
    }

    #[test]
    fn a_directory_already_used_is_refused_and_starts_nothing() {
        let root = Scratch::new("worker-fresh");
        let dir = root.join("worker-1");
        let (run, _lifeline) = session(&dir, "exit 0");
        ended(&dir, &run);
        let (reader, _writer) = std::io::pipe().unwrap();
        assert!(launch(&dir, "/bin/sh", &[], &REPOSITORY.parse().unwrap(), reader).is_err());
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
        HeadlessSessions.stop(&run).unwrap();
        assert_eq!(ended(&dir, &run), Session::Ended(None));
        assert_eq!(HeadlessSessions.stop(&run), Ok(()));
    }

    #[test]
    fn a_process_that_merely_has_the_recorded_id_is_not_the_session() {
        let root = Scratch::new("worker-stranger");
        let dir = root.join("worker-1");
        let (run, _lifeline) = session(&dir, "sleep 30");
        let stranger = WorkerRun { transcript: "/elsewhere/transcript.jsonl".parse().unwrap(), ..run.clone() };
        assert_eq!(is_alive(&stranger), Ok(false));
        assert_eq!(HeadlessSessions.stop(&stranger), Ok(()));
        assert_eq!(state_in(&dir, &run), Ok(Session::Running));
        assert!(!dir.join(STATUS).exists());
        HeadlessSessions.stop(&run).unwrap();
    }
}
