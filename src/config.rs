//! The typed `[machine.hundegger]` config sub-section.
//!
//! In a factory config, a Hundegger machine looks like:
//!
//! ```toml
//! [[machine]]
//! id     = "hundegger-1"
//! driver = "hundegger"
//! [machine.identification]            # standard OPC-UA Machinery nameplate
//! manufacturer = "Hundegger"
//! model        = "K2"
//! [machine.hundegger]                 # this struct — driver-specific
//! dispatch_dir = "/mnt/cambium_import"
//! format       = "btlx"
//! ```
//!
//! The gateway hands the `[machine.hundegger]` table to this driver verbatim.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Output format handed to the machine's controller.
///
/// **BTLx** is the primary target: open, machine-agnostic, validated against the
/// published XSD, and consumed by every established wood CAD and by Cambium /
/// NC-HOPS. **Bvx** is Hundegger's own format (also XML), used by the panel line
/// (SPM-2 / PBA / SIP) and the SC3 / Cambium saw — added only when a specific
/// machine needs it. *(Bvx serialisation is not yet implemented.)*
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Btlx,
    Bvx,
}

/// Hundegger-specific machine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HundeggerConfig {
    /// Directory the controller ingests parts from — the Cambium import folder
    /// (or a watched hand-off folder for an intermediate post-processor).
    ///
    /// **The exact ingest mechanism is unconfirmed** (watched hot folder vs.
    /// manual import vs. API); see the driver's `run_job`. Writing a valid file
    /// here is the current best-known hand-off.
    pub dispatch_dir: PathBuf,
    /// Directory the machine controller writes its job status log into — we poll it
    /// to close the loop (job started / completed / error).
    ///
    /// **The real Cambium log format is unconfirmed** — see [`crate::status`]. Until
    /// we have a sample, the log is parsed as the format the bundled simulator writes
    /// (`btlx sim`), isolated behind one parser we swap when the real format is known.
    pub status_dir: PathBuf,
    /// Serialised output format. See [`OutputFormat`].
    pub format: OutputFormat,
}

impl Default for HundeggerConfig {
    fn default() -> Self {
        Self {
            dispatch_dir: PathBuf::from("/mnt/cambium_import"),
            status_dir: PathBuf::from("/mnt/cambium_status"),
            format: OutputFormat::Btlx,
        }
    }
}

impl HundeggerConfig {
    /// File extension for the configured [`OutputFormat`].
    pub fn extension(&self) -> &'static str {
        match self.format {
            OutputFormat::Btlx => "btlx",
            OutputFormat::Bvx => "bvx",
        }
    }
}
