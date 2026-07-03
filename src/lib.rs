//! # factory-btlx
//!
//! The Hundegger timber-CNC machine driver for the `factory-` family. Two parts:
//!
//! - [`btlx`] — a serialiser for **BTLx**, the open, machine-agnostic timber
//!   fabrication interchange format ([design2machine](https://www.design2machine.com/)).
//!   This is the reusable, machine-independent core: turn a parametric part
//!   description into a `.btlx` file that validates against the published schema.
//! - [`driver`] — [`Hundegger`], which implements
//!   [`factory_machine_model::MachineDriver`]: it dispatches a BTLx payload to the
//!   machine's controller and closes the loop by reading the [`status`] log back.
//! - [`sim`] — a stand-in controller so the whole dispatch→telemetry loop runs and is
//!   tested without a real machine (`btlx sim`).
//!
//! The gateway composes this crate when a factory's config declares a machine
//! with `driver = "hundegger"`.
//!
//! ## Scope
//!
//! The BTLx model covers the common processings (Lap, JackRafterCut, Drilling,
//! Mortise, Tenon), validated against the real XSD. The driver's dispatch + telemetry
//! loop is complete and tested via the simulator; the one genuine unknown left is
//! Cambium's real ingest mechanism + status-log format, isolated behind
//! [`status::parse`] (see the module docs).

pub mod btlx;
pub mod config;
pub mod driver;
pub mod inspect;
pub mod sim;
pub mod status;

pub use config::{HundeggerConfig, OutputFormat};
pub use driver::Hundegger;

/// The driver `kind` string this crate handles — matches `driver = "..."` in config.
pub const KIND: &str = "hundegger";
