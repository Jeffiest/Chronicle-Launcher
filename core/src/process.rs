//! Running the port's programs: each line they print goes to a callback as it comes, and to a log file.

use serde::Serialize;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Mutex};
use std::thread;
use std::time::Duration;

/// How a program ended: its exit code, or the signal that stopped it (POSIX), or neither when it was killed on
/// Windows.
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct Ended {
    pub code: Option<i32>,
    pub signal: Option<i32>,
}

impl Ended {
    pub fn success(&self) -> bool {
        self.code == Some(0)
    }
}

impl From<std::process::ExitStatus> for Ended {
    fn from(status: std::process::ExitStatus) -> Self {
        #[cfg(unix)]
        let signal = std::os::unix::process::ExitStatusExt::signal(&status);
        #[cfg(not(unix))]
        let signal = None;
        Ended {
            code: status.code(),
            signal,
        }
    }
}

/// Starts `command` with its output piped. The caller keeps the child, to wait on or kill.
pub fn spawn(mut command: Command) -> io::Result<Child> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        // No console window for the port's command-line programs.
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    command.spawn()
}

/// Reads the child's stdout and stderr until both close, passing each line to `on_line` (in the order they arrive)
/// and writing it to `log`. Returns once the child has exited. The child stays behind its lock between polls, so
/// another thread can kill it (`kill`).
pub fn follow(child: &Mutex<Child>, log: &Path, mut on_line: impl FnMut(&str)) -> io::Result<Ended> {
    let (send, lines) = mpsc::channel::<String>();
    let mut readers = Vec::new();
    let streams = {
        let mut child = child.lock().unwrap_or_else(|e| e.into_inner());
        [
            child
                .stdout
                .take()
                .map(|s| Box::new(s) as Box<dyn io::Read + Send>),
            child
                .stderr
                .take()
                .map(|s| Box::new(s) as Box<dyn io::Read + Send>),
        ]
    };
    for stream in streams.into_iter().flatten() {
        let send = send.clone();
        readers.push(thread::spawn(move || {
            for line in BufReader::new(stream).split(b'\n').map_while(Result::ok) {
                let line = String::from_utf8_lossy(&line).trim_end_matches('\r').to_string();
                if send.send(line).is_err() {
                    break;
                }
            }
        }));
    }
    drop(send);
    if let Some(parent) = log.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::File::create(log)?;
    for line in lines {
        writeln!(file, "{line}")?;
        on_line(&line);
    }
    for reader in readers {
        let _ = reader.join();
    }
    loop {
        if let Some(status) = child.lock().unwrap_or_else(|e| e.into_inner()).try_wait()? {
            return Ok(status.into());
        }
        thread::sleep(Duration::from_millis(50));
    }
}

/// Stops a child that `follow` is reading; `follow` then returns how it ended.
pub fn kill(child: &Mutex<Child>) {
    let _ = child.lock().unwrap_or_else(|e| e.into_inner()).kill();
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn streams_both_outputs_and_keeps_a_log() {
        let dir = tempfile::tempdir().unwrap();
        let mut command = Command::new("sh");
        command.args(["-c", "echo one; echo two >&2; exit 3"]);
        let child = Mutex::new(spawn(command).unwrap());
        let mut seen = Vec::new();
        let ended = follow(&child, &dir.path().join("logs/run.log"), |l| {
            seen.push(l.to_string())
        })
        .unwrap();
        seen.sort();
        assert_eq!(seen, ["one", "two"]);
        assert_eq!(
            ended,
            Ended {
                code: Some(3),
                signal: None
            }
        );
        let log = std::fs::read_to_string(dir.path().join("logs/run.log")).unwrap();
        assert!(log.contains("one") && log.contains("two"));
    }

    #[test]
    fn a_killed_program_says_so() {
        let dir = tempfile::tempdir().unwrap();
        let mut command = Command::new("sh");
        command.args(["-c", "kill -SEGV $$"]);
        let child = Mutex::new(spawn(command).unwrap());
        let ended = follow(&child, &dir.path().join("run.log"), |_| {}).unwrap();
        assert_eq!(ended.signal, Some(11));
        assert!(!ended.success());
    }

    #[test]
    fn a_follow_can_be_cancelled_from_another_thread() {
        let dir = tempfile::tempdir().unwrap();
        let mut command = Command::new("sh");
        command.args(["-c", "echo started; exec sleep 30"]);
        let child = std::sync::Arc::new(Mutex::new(spawn(command).unwrap()));
        let other = child.clone();
        let killer = thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            kill(&other);
        });
        let started = std::time::Instant::now();
        let ended = follow(&child, &dir.path().join("run.log"), |_| {}).unwrap();
        killer.join().unwrap();
        assert!(started.elapsed() < Duration::from_secs(10));
        assert!(!ended.success());
    }
}
