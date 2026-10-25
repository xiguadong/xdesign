use std::process::Command;
use std::time::{Duration, Instant};
use xdesign::simple_flow::{checker_timeout, run_command_with_timeout};

#[test]
fn runaway_checker_is_killed_at_its_deadline() {
    let started = Instant::now();
    let outcome =
        run_command_with_timeout(Command::new("sleep").arg("30"), Duration::from_millis(300))
            .unwrap();
    assert!(outcome.timed_out);
    assert!(!outcome.success);
    assert!(started.elapsed() < Duration::from_secs(5));
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
