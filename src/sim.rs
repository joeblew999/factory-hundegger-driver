//! A stand-in Hundegger controller, for developing and testing the driver loop
//! **without a real machine.**
//!
//! It watches the dispatch directory for `.btlx` files (the driver's hand-off) and
//! appends status-log lines — `running`, then `completed` (or `failed` on a broken
//! file) — to `<status_dir>/status.jsonl`, mimicking the feedback a real controller
//! gives. That lets the whole dispatch→telemetry loop run and be tested end-to-end on
//! any laptop, no hardware. It is **not** a real machine: the status format is ours
//! (see [`crate::status`]); when we get a real Cambium log, the driver's parser
//! changes and the simulator can be retired.

use std::collections::HashSet;
use std::io::Write;
use std::path::Path;

use crate::inspect::inspect_str;
use crate::status::{self, JobState, STATUS_LOG, StatusEntry};

/// Result of one simulator pass.
#[derive(Debug, Default, PartialEq)]
pub struct SimRun {
    pub completed: Vec<String>,
    pub failed: Vec<String>,
}

/// Process every `.btlx` in `dispatch_dir` not already terminal in the status log,
/// appending `running` then `completed`/`failed` lines to `<status_dir>/status.jsonl`.
/// Idempotent: a job already completed/failed in the log is skipped, so it's safe to
/// call in a loop.
pub fn process_pending(dispatch_dir: &Path, status_dir: &Path) -> anyhow::Result<SimRun> {
    std::fs::create_dir_all(status_dir)?;
    let log_path = status_dir.join(STATUS_LOG);
    let existing = std::fs::read_to_string(&log_path).unwrap_or_default();
    let done: HashSet<String> = status::parse(&existing)
        .into_iter()
        .filter(|e| e.state.is_terminal())
        .map(|e| e.job_id)
        .collect();

    let mut run = SimRun::default();
    let mut out = String::new();
    let mut push = |e: StatusEntry| {
        out.push_str(&e.to_line());
        out.push('\n');
    };

    if dispatch_dir.exists() {
        let mut files: Vec<_> = std::fs::read_dir(dispatch_dir)?.flatten().collect();
        files.sort_by_key(|e| e.file_name());
        for de in files {
            let path = de.path();
            if path.extension().and_then(|s| s.to_str()) != Some("btlx") {
                continue;
            }
            let Some(job_id) = path.file_stem().and_then(|s| s.to_str()).map(str::to_owned) else {
                continue;
            };
            if job_id.is_empty() || done.contains(&job_id) {
                continue;
            }

            push(StatusEntry::new(&job_id, JobState::Running));
            match std::fs::read_to_string(&path)
                .ok()
                .and_then(|xml| inspect_str(&xml).ok())
            {
                Some(report) => {
                    push(
                        StatusEntry::new(&job_id, JobState::Completed)
                            .with_parts(report.parts as u32),
                    );
                    run.completed.push(job_id);
                }
                None => {
                    push(
                        StatusEntry::new(&job_id, JobState::Failed).with_detail("unreadable .btlx"),
                    );
                    run.failed.push(job_id);
                }
            }
        }
    }

    if !out.is_empty() {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        f.write_all(out.as_bytes())?;
    }
    Ok(run)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processes_a_btlx_and_writes_completed() {
        let base = std::env::temp_dir().join("factory-btlx-sim-test");
        let dispatch = base.join("in");
        let statusd = base.join("status");
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&dispatch).unwrap();

        // A valid one-part BTLx, dropped in the dispatch dir as job-1.btlx.
        let xml = crate::btlx::to_xml(&crate::btlx::model::Btlx::new(
            crate::btlx::model::Project::new(
                "job-1",
                vec![crate::btlx::model::Part::new(3000.0, 160.0, 80.0)],
            ),
        ))
        .unwrap();
        std::fs::write(dispatch.join("job-1.btlx"), xml).unwrap();
        // A broken file → should be reported failed.
        std::fs::write(dispatch.join("broken.btlx"), "<not xml").unwrap();

        let run = process_pending(&dispatch, &statusd).unwrap();
        assert_eq!(run.completed, ["job-1"]);
        assert_eq!(run.failed, ["broken"]);

        // Re-running is idempotent — nothing new is processed.
        let again = process_pending(&dispatch, &statusd).unwrap();
        assert!(again.completed.is_empty() && again.failed.is_empty());

        // The log records the completion with the part count.
        let log = std::fs::read_to_string(statusd.join(STATUS_LOG)).unwrap();
        let latest = status::latest_by_job(&status::parse(&log));
        assert_eq!(latest["job-1"].state, JobState::Completed);
        assert_eq!(latest["job-1"].parts, Some(1));
    }
}
