use std::process::Command;
use std::time::{Duration, Instant};
use xdesign::simple_flow::{checker_timeout, run_command_with_timeout};

/// Slack above a short deadline within which a bounded implementation must
/// return, even when descendants deliberately hold the inherited pipes open
/// for two seconds. The tests never wait for those descendants.
const BOUNDED_SLACK: Duration = Duration::from_millis(1500);

fn python(script: &str) -> Command {
    let mut command = Command::new("python3");
    command.arg("-c").arg(script);
    command
}

fn assert_bounded_elapsed(elapsed: Duration) {
    assert!(
        elapsed < BOUNDED_SLACK,
        "elapsed {elapsed:?} exceeded the bounded-deadline slack {BOUNDED_SLACK:?}"
    );
}

#[test]
fn runaway_checker_is_killed_at_its_deadline() {
    let started = Instant::now();
    let outcome =
        run_command_with_timeout(Command::new("sleep").arg("30"), Duration::from_millis(300))
            .unwrap();
    assert!(outcome.timed_out);
    assert!(!outcome.success);
    assert_bounded_elapsed(started.elapsed());
}

#[test]
fn checker_within_its_deadline_collects_output() {
    let outcome = run_command_with_timeout(
        Command::new("printf").arg("checker output"),
        Duration::from_secs(30),
    )
    .unwrap();
    assert!(!outcome.timed_out);
    assert!(outcome.success);
    assert_eq!(outcome.stdout, b"checker output");
    assert!(outcome.stderr.is_empty());
}

#[test]
fn checker_timeout_defaults_and_env_override() {
    assert_eq!(checker_timeout(), Duration::from_secs(600));
    // The override is read per call; restore it before returning so parallel
    // tests in this process are unaffected.
    std::env::set_var("XDESIGN_CHECKER_TIMEOUT_SECS", "7");
    assert_eq!(checker_timeout(), Duration::from_secs(7));
    std::env::remove_var("XDESIGN_CHECKER_TIMEOUT_SECS");
}

/// The direct child exits at once but a two-second descendant inherits and
/// holds its stdout/stderr write ends. A bounded checker must expire the
/// deadline instead of waiting for the descendant to release the pipes, and it
/// must not report the command as successful merely because the direct child
/// exited with status zero.
#[test]
fn fast_exiting_checker_cannot_leave_descendants_holding_the_deadline_open() {
    let started = Instant::now();
    let outcome = run_command_with_timeout(
        &mut python("import subprocess; subprocess.Popen(['sleep', '2']); print('done')"),
        Duration::from_millis(300),
    )
    .unwrap();
    let elapsed = started.elapsed();
    assert_bounded_elapsed(elapsed);
    assert!(
        outcome.timed_out,
        "descendant-held pipes must expire the deadline, not report success"
    );
    assert!(!outcome.success);
}

/// A long-running parent that also leaves a two-second descendant must be
/// killed with its whole process group at the deadline.
#[test]
fn long_running_checker_tree_is_killed_at_the_deadline() {
    let started = Instant::now();
    let outcome = run_command_with_timeout(
        &mut python("import subprocess, time; subprocess.Popen(['sleep', '2']); time.sleep(5)"),
        Duration::from_millis(200),
    )
    .unwrap();
    let elapsed = started.elapsed();
    assert_bounded_elapsed(elapsed);
    assert!(outcome.timed_out);
    assert!(!outcome.success);
}

/// A normal command that closes its pipes must still return its full captured
/// output and success exactly as before the process-group change.
#[test]
fn normal_checker_still_captures_stdout_and_stderr() {
    let outcome = run_command_with_timeout(
        &mut python("import sys; print('normal stdout'); print('normal stderr', file=sys.stderr)"),
        Duration::from_secs(30),
    )
    .unwrap();
    assert!(!outcome.timed_out);
    assert!(outcome.success);
    assert_eq!(outcome.stdout, b"normal stdout\n");
    assert_eq!(outcome.stderr, b"normal stderr\n");
}

/// Output larger than the OS pipe buffer exercises the non-blocking drain:
/// the child blocks on write until the checker reads, so the deadline loop
/// must keep consuming both streams without dropping bytes.
#[test]
fn normal_checker_output_larger_than_the_pipe_buffer_is_captured_fully() {
    let outcome = run_command_with_timeout(
        &mut python("import sys; sys.stdout.write('x' * 100000); sys.stdout.flush()"),
        Duration::from_secs(30),
    )
    .unwrap();
    assert!(!outcome.timed_out);
    assert!(outcome.success);
    assert_eq!(outcome.stdout.len(), 100_000);
    assert!(outcome.stdout.iter().all(|byte| *byte == b'x'));
    assert!(outcome.stderr.is_empty());
}
