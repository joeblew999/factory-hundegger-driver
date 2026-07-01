//! `Hundegger` — the [`MachineDriver`] implementation for Hundegger timber-CNC
//! machines. The gateway only ever sees the standard `factory-machine-model`
//! contract; everything Hundegger-specific lives here.
//!
//! The job payload is a serialised **BTLx** document (see [`crate::btlx`]) — the
//! same role the cut-list CSV plays for Howick. `run_job` hands that file to the
//! machine's controller by writing it into the configured dispatch directory.
//!
//! ## Known unknowns (need a real shop / Hundegger — see the repo README)
//!
//! - **Ingest mechanism.** Whether Cambium watches the dispatch folder, needs a
//!   manual import, or exposes an API is unconfirmed. `run_job` writes a valid
//!   file to `dispatch_dir` as the best-known hand-off; swap in the real
//!   mechanism once known.
//! - **Telemetry.** [`poll_telemetry`](Hundegger::poll_telemetry) reports only a
//!   dispatch counter. Real machine state / job feedback needs a sample of the
//!   controller's status-log format, which we don't have yet.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use factory_machine_model::{
    Identification, JobOrder, MachineDescriptor, MachineDriver, MachineryItemState, Telemetry,
    TelemetryField, Value, ValueKind,
};

use crate::config::HundeggerConfig;

/// Telemetry BrowseName exposed under `Machines/<id>/Telemetry/`.
///
/// Placeholder until we can parse real controller logs — see the module note.
pub const JOBS_DISPATCHED: &str = "JobsDispatched";

/// Driver for one Hundegger machine reachable from this host.
pub struct Hundegger {
    machine_id: String,
    identification: Identification,
    config: HundeggerConfig,
    jobs_dispatched: AtomicU64,
    running: AtomicBool,
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
            running: AtomicBool::new(false),
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
}

impl MachineDriver for Hundegger {
    fn descriptor(&self) -> MachineDescriptor {
        MachineDescriptor {
            machine_id: self.machine_id.clone(),
            kind: crate::KIND.to_owned(),
            identification: self.identification.clone(),
            telemetry: vec![TelemetryField::new(JOBS_DISPATCHED, ValueKind::UInt, None)],
        }
    }

    async fn state(&self) -> MachineryItemState {
        if self.running.load(Ordering::Relaxed) {
            MachineryItemState::Executing
        } else {
            MachineryItemState::NotExecuting
        }
    }

    async fn run_job(&self, job: &JobOrder) -> anyhow::Result<()> {
        let payload = job
            .payload()
            .ok_or_else(|| anyhow::anyhow!("job {} carries no BTLx payload", job.job_order_id))?;

        self.running.store(true, Ordering::Relaxed);
        let result = self.dispatch(&job.job_order_id, payload).await;
        self.running.store(false, Ordering::Relaxed);
        let path = result?;
        tracing::info!(job = %job.job_order_id, path = %path.display(), "dispatched BTLx to machine");

        self.jobs_dispatched.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn poll_telemetry(&self) -> anyhow::Result<Telemetry> {
        let mut t = Telemetry::new();
        t.insert(
            JOBS_DISPATCHED.to_owned(),
            Value::UInt(self.jobs_dispatched.load(Ordering::Relaxed)),
        );
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btlx::{model::*, to_xml};

    fn driver() -> Hundegger {
        let cfg = HundeggerConfig {
            dispatch_dir: std::env::temp_dir().join("factory-hundegger-test"),
            ..Default::default()
        };
        Hundegger::new("hundegger-1", Identification::new("Hundegger", "K2"), cfg)
    }

    #[test]
    fn descriptor_is_standard_plus_hundegger_telemetry() {
        let d = driver().descriptor();
        assert_eq!(d.kind, crate::KIND);
        assert_eq!(d.identification.manufacturer, "Hundegger");
        let names: Vec<&str> = d.telemetry.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, [JOBS_DISPATCHED]);
    }

    #[tokio::test]
    async fn dispatches_a_btlx_job_to_the_machine() {
        // Produce a real BTLx payload via our own serialiser, then dispatch it.
        let part = Part::new(3000.0, 160.0, 80.0).with_processings(vec![Processing::Drilling(
            Drilling::new("bolt-hole", 1, RefPlane::Global(3), 500.0, 80.0, 80.0, 12.0),
        )]);
        let xml = to_xml(&Btlx::new(Project::new("job-42", vec![part]))).unwrap();

        let d = driver();
        assert_eq!(d.state().await, MachineryItemState::NotExecuting);
        let job = JobOrder::with_payload("job-42", "BtlxDoc", xml.into_bytes());
        d.run_job(&job).await.unwrap();

        let written = std::fs::read_to_string(d.config.dispatch_dir.join("job-42.btlx")).unwrap();
        assert!(
            written.contains("<Drilling"),
            "the BTLx reaches the machine intact"
        );

        let t = d.poll_telemetry().await.unwrap();
        assert_eq!(t.get(JOBS_DISPATCHED), Some(&Value::UInt(1)));
    }
}
