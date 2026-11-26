use std::time::Duration;
use xtask::workspace::{capture_command, pause_for, terminate_active_children};

#[test]
fn terminating_active_children_stops_a_stalled_command() {
    let directory = std::env::temp_dir();
    let worker = std::thread::spawn(move || capture_command(&directory, "sleep", &["60"], &[0]));
    pause_for(Duration::from_millis(500));
    terminate_active_children();
    assert!(worker.join().unwrap().is_err());
}
