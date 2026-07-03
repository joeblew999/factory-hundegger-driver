//! `Hundegger` — the [`MachineDriver`] implementation for Hundegger timber-CNC
//! machines. The gateway only ever sees the standard `factory-machine-model`
//! contract; everything Hundegger-specific lives here.
//!
//! The job payload is a serialised **BTLx** document (see [`crate::btlx`]) — the
//! same role the cut-list CSV plays for Howick. `run_job` hands that file to the
//! machine's controller by writing it into the configured dispatch directory.
//!
//! The loop is closed by polling the controller's **status log** (see
//! [`crate::status`]): `run_job` dispatches a BTLx file, and `state` / `poll_telemetry`
//! read `status_dir` to learn what the machine did. With no real machine to hand, the
//! bundled simulator ([`crate::sim`], `btlx sim`) plays the controller — so the whole
//! loop runs and is tested end-to-end here.
//!
//! ## The one genuine unknown (needs a real shop / Hundegger)
//!
//! - **Ingest mechanism.** Whether Cambium watches the dispatch folder, needs a
//!   manual import, or exposes an API is unconfirmed. `run_job` writes a valid file to
//!   `dispatch_dir` as the best-known hand-off; swap in the real mechanism once known.
//! - **Status-log format.** The real Cambium log format is unconfirmed, so the loop
//!   parses the simulator's format via [`crate::status::parse`] — the single place to
//!   change when a real sample arrives. Everything above it is done.

use std::sync::atomic::{AtomicU64, Ordering};

use factory_machine_model::{
    Identification, JobOrder, MachineDescriptor, MachineDriver, MachineryItemState, Telemetry,
    TelemetryField, Value, ValueKind,
};

use crate::config::HundeggerConfig;
use crate::status::{self, JobState, STATUS_LOG};

/// Telemetry BrowseNames exposed under `Machines/<id>/Telemetry/`.
pub const JOBS_DISPATCHED: &str = "JobsDispatched";
pub const JOBS_COMPLETED: &str = "JobsCompleted";
pub const JOBS_FAILED: &str = "JobsFailed";

/// Driver for one Hundegger machine reachable from this host.
pub struct Hundegger {
    machine_id: String,
    identification: Identification,
    config: HundeggerConfig,
    jobs_dispatched: AtomicU64,
}

impl Hundegger {
    /// Build from a machine id, its standard nameplate, and the typed config.
    pub fn new(
        machine_id: impl Into<String>,
        identification: Identification,
        config: HundeggerConfig,
    ) -> Self {
        Self {
            machine_id: machine_id.into(),
            identification,
            config,
            jobs_dispatched: AtomicU64::new(0),
        }
    }

    /// Write a payload into the dispatch dir as `<job_id>.<ext>`, returning the
    /// path written. Creates the directory if needed.
    async fn dispatch(&self, job_id: &str, payload: &[u8]) -> anyhow::Result<std::path::PathBuf> {
        tokio::fs::create_dir_all(&self.config.dispatch_dir).await?;
        let path = self
            .config
            .dispatch_dir
            .join(format!("{job_id}.{}", self.config.extension()));
        tokio::fs::write(&path, payload).await?;
        Ok(path)
    }

    /// Read + parse the machine's status log (empty if not written yet).
    async fn status(&self) -> Vec<status::StatusEntry> {
        let log = tokio::fs::read_to_string(self.config.status_dir.join(STATUS_LOG))
            .await
            .unwrap_or_default();
        status::parse(&log)
    }
}

impl MachineDriver for Hundegger {
    fn descriptor(&self) -> MachineDescriptor {
        MachineDescriptor {
            machine_id: self.machine_id.clone(),
            kind: crate::KIND.to_owned(),
            identification: self.identification.clone(),
            telemetry: vec![
                TelemetryField::new(JOBS_DISPATCHED, ValueKind::UInt, None),
                TelemetryField::new(JOBS_COMPLETED, ValueKind::UInt, None),
                TelemetryField::new(JOBS_FAILED, ValueKind::UInt, None),
            ],
        }
    }

    /// `Executing` if any job is `running` in the status log, else `NotExecuting`.
    async fn state(&self) -> MachineryItemState {
        let latest = status::latest_by_job(&self.status().await);
        if latest.values().any(|e| e.state == JobState::Running) {
            MachineryItemState::Executing
        } else {
            MachineryItemState::NotExecuting
        }
    }

    async fn run_job(&self, job: &JobOrder) -> anyhow::Result<()> {
        let payload = job
            .payload()
            .ok_or_else(|| anyhow::anyhow!("job {} carries no BTLx payload", job.job_order_id))?;

        let path = self.dispatch(&job.job_order_id, payload).await?;
        tracing::info!(job = %job.job_order_id, path = %path.display(), "dispatched BTLx to machine");

        self.jobs_dispatched.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Dispatch counter (what we sent) plus completed/failed counts read back from the
    /// machine's status log (what it actually did).
    async fn poll_telemetry(&self) -> anyhow::Result<Telemetry> {
        let latest = status::latest_by_job(&self.status().await);
        let count = |s: JobState| latest.values().filter(|e| e.state == s).count() as u64;

        let mut t = Telemetry::new();
        t.insert(
            JOBS_DISPATCHED.to_owned(),
            Value::UInt(self.jobs_dispatched.load(Ordering::Relaxed)),
        );
        t.insert(
            JOBS_COMPLETED.to_owned(),
            Value::UInt(count(JobState::Completed)),
        );
        t.insert(JOBS_FAILED.to_owned(), Value::UInt(count(JobState::Failed)));
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btlx::{model::*, to_xml};

    /// A driver with isolated dispatch + status dirs under a per-test base.
    fn driver(name: &str) -> Hundegger {
        let base = std::env::temp_dir().join("factory-btlx-driver").join(name);
        let _ = std::fs::remove_dir_all(&base);
        let cfg = HundeggerConfig {
            dispatch_dir: base.join("in"),
            status_dir: base.join("status"),
            ..Default::default()
        };
        Hundegger::new("hundegger-1", Identification::new("Hundegger", "K2"), cfg)
    }

    fn sample(job: &str) -> Vec<u8> {
        let part = Part::new(3000.0, 160.0, 80.0).with_processings(vec![Processing::Drilling(
            Drilling::new("bolt-hole", 1, RefPlane::Global(3), 500.0, 80.0, 80.0, 12.0),
        )]);
        to_xml(&Btlx::new(Project::new(job, vec![part])))
            .unwrap()
            .into_bytes()
    }

    #[test]
    fn descriptor_is_standard_plus_hundegger_telemetry() {
        let d = driver("descriptor").descriptor();
        assert_eq!(d.kind, crate::KIND);
        assert_eq!(d.identification.manufacturer, "Hundegger");
        let names: Vec<&str> = d.telemetry.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, [JOBS_DISPATCHED, JOBS_COMPLETED, JOBS_FAILED]);
    }

    #[tokio::test]
    async fn dispatches_a_btlx_job_to_the_machine() {
        let d = driver("dispatch");
        let job = JobOrder::with_payload("job-42", "BtlxDoc", sample("job-42"));
        d.run_job(&job).await.unwrap();

        let written = std::fs::read_to_string(d.config.dispatch_dir.join("job-42.btlx")).unwrap();
        assert!(
            written.contains("<Drilling"),
            "the BTLx reaches the machine intact"
        );
        assert_eq!(
            d.poll_telemetry().await.unwrap().get(JOBS_DISPATCHED),
            Some(&Value::UInt(1))
        );
    }

    /// The full loop, no hardware: driver dispatches → simulator plays the controller →
    /// driver reads the status log back as telemetry + machine state.
    #[tokio::test]
    async fn full_dispatch_to_telemetry_loop_via_simulator() {
        let d = driver("loop");

        // Idle before anything runs.
        assert_eq!(d.state().await, MachineryItemState::NotExecuting);

        // Dispatch two jobs.
        d.run_job(&JobOrder::with_payload("job-a", "BtlxDoc", sample("job-a")))
            .await
            .unwrap();
        d.run_job(&JobOrder::with_payload("job-b", "BtlxDoc", sample("job-b")))
            .await
            .unwrap();

        // Nothing has run them yet: dispatched=2, completed=0.
        let t = d.poll_telemetry().await.unwrap();
        assert_eq!(t.get(JOBS_DISPATCHED), Some(&Value::UInt(2)));
        assert_eq!(t.get(JOBS_COMPLETED), Some(&Value::UInt(0)));

        // The simulator (standing in for Cambium) processes the dispatch dir.
        let run =
            crate::sim::process_pending(&d.config.dispatch_dir, &d.config.status_dir).unwrap();
        assert_eq!(run.completed.len(), 2);

        // The driver now reads the machine's feedback back as telemetry.
        let t = d.poll_telemetry().await.unwrap();
        assert_eq!(t.get(JOBS_COMPLETED), Some(&Value::UInt(2)));
        assert_eq!(t.get(JOBS_FAILED), Some(&Value::UInt(0)));
        assert_eq!(d.state().await, MachineryItemState::NotExecuting);
    }
}
