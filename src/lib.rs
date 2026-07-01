//! # factory-hundegger-driver
//!
//! The Hundegger timber-CNC machine driver for the `factory-` family. Two parts:
//!
//! - [`btlx`] — a serialiser for **BTLx**, the open, machine-agnostic timber
//!   fabrication interchange format ([design2machine](https://www.design2machine.com/)).
//!   This is the reusable, machine-independent core: turn a parametric part
//!   description into a `.btlx` file that validates against the published schema.
//! - [`driver`] — [`Hundegger`], which implements
//!   [`factory_machine_model::MachineDriver`], handing a BTLx payload to the
//!   machine's controller (Cambium / NC-HOPS) and reporting state.
//!
//! The gateway composes this crate when a factory's config declares a machine
//! with `driver = "hundegger"`.
//!
//! ## Scope
//!
//! This is an early scaffold. The BTLx model covers the document/part structure
//! and [`Drilling`](btlx::Drilling) as the first processing; the machine-facing
//! dispatch and telemetry are stubbed pending a real sample from a shop (see the
//! repository README for the exact open questions).

pub mod btlx;
pub mod config;
pub mod driver;

pub use config::{HundeggerConfig, OutputFormat};
pub use driver::Hundegger;

/// The driver `kind` string this crate handles — matches `driver = "..."` in config.
pub const KIND: &str = "hundegger";
