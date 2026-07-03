//! Job status — the machine's feedback loop.
//!
//! The driver dispatches a BTLx file, then polls the controller's status log to learn
//! what happened: queued → running → completed / failed. That log closes the loop so
//! the [digital twin](crate::driver) knows what actually got made.
//!
//! **We don't have a real Cambium status-log sample yet.** So [`parse`] reads the
//! simple JSONL format the bundled simulator ([`crate::sim`]) writes. **This parser is
//! the single place to change** when a real log format arrives — the driver's
//! telemetry and the gateway above it don't move. That isolation is the whole point:
//! the one genuine unknown (Cambium's log format) sits behind one function.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A job's state on the machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobState {
    Queued,
    Running,
    Completed,
    Failed,
}

impl JobState {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobState::Queued => "queued",
            JobState::Running => "running",
            JobState::Completed => "completed",
            JobState::Failed => "failed",
        }
    }
    /// A job that will not change again.
    pub fn is_terminal(&self) -> bool {
        matches!(self, JobState::Completed | JobState::Failed)
    }
}

/// One line of the status log — a state transition the machine reported.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusEntry {
    #[serde(rename = "job")]
    pub job_id: String,
    pub state: JobState,
    /// Parts finished (reported on completion), if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parts: Option<u32>,
    /// Error or info detail, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

impl StatusEntry {
    pub fn new(job_id: impl Into<String>, state: JobState) -> Self {
        Self {
            job_id: job_id.into(),
            state,
            parts: None,
            detail: None,
        }
    }
    pub fn with_parts(mut self, parts: u32) -> Self {
        self.parts = Some(parts);
        self
    }
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }
    /// Render as one JSONL line — what the machine (or simulator) appends to the log.
    pub fn to_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// The status-log file name inside the configured status directory.
pub const STATUS_LOG: &str = "status.jsonl";

/// Parse a status log (append-only JSONL) into entries, skipping blank/garbage lines.
///
/// **Swap this when the real Cambium log format is known** — nothing else changes.
pub fn parse(log: &str) -> Vec<StatusEntry> {
    log.lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str::<StatusEntry>(l).ok())
        .collect()
}

/// The latest state per job (the log is append-only, so the last line for a job wins).
pub fn latest_by_job(entries: &[StatusEntry]) -> BTreeMap<String, StatusEntry> {
    let mut m = BTreeMap::new();
    for e in entries {
        m.insert(e.job_id.clone(), e.clone());
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrips_a_status_line() {
        let e = StatusEntry::new("job-42", JobState::Completed).with_parts(3);
        let line = e.to_line();
        assert!(line.contains("\"job\":\"job-42\""));
        assert!(line.contains("\"state\":\"completed\""));
        let back = parse(&line);
        assert_eq!(back, vec![e]);
    }

    #[test]
    fn latest_wins_and_garbage_is_skipped() {
        let log = "\
{\"job\":\"a\",\"state\":\"running\"}
not json
{\"job\":\"a\",\"state\":\"completed\",\"parts\":2}
{\"job\":\"b\",\"state\":\"failed\",\"detail\":\"tool timeout\"}
";
        let latest = latest_by_job(&parse(log));
        assert_eq!(latest["a"].state, JobState::Completed);
        assert_eq!(latest["a"].parts, Some(2));
        assert_eq!(latest["b"].state, JobState::Failed);
        assert_eq!(latest["b"].detail.as_deref(), Some("tool timeout"));
    }
}
