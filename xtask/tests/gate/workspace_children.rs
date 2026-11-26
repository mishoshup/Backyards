use std::time::Duration;
use xtask::workspace::{capture_command, pause_for, terminate_active_children};

const STALL_GRACE: Duration = Duration::from_millis(500);
const STALLED_COMMAND_SECONDS: &str = "60";

#[test]
fn terminating_active_children_stops_a_stalled_command() {
    let directory = std::env::temp_dir();
    let worker = std::thread::spawn(move || {
        capture_command(&directory, "sleep", &[STALLED_COMMAND_SECONDS], &[0])
    });
    pause_for(STALL_GRACE);
    terminate_active_children();
    assert!(worker.join().is_ok_and(|result| result.is_err()));
}
