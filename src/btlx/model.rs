//! The BTLx data model — Rust structs mirroring the **BTLx 2.3.1** XML schema
//! ([`fixtures/schema/BTLx_2_3_1.xsd`], from [design2machine](https://www.design2machine.com/)).
//!
//! BTLx is the open, machine-agnostic interchange standard for timber
//! fabrication (maintained by SEMA + CadWork). A file describes each *part* (a
//! raw stock piece) parametrically as a list of *processings* — cuts, drillings,
//! mortises, tenons — each located on a **reference plane**. The Hundegger
//! machine's own software (Cambium / NC-HOPS) turns this into tool paths.
//!
//! This module models the subset we can emit today. It is deliberately not the
//! whole schema (40+ processing types): we start with [`Drilling`], the simplest
//! well-defined processing, and grow the [`Processing`] enum as real sample files
//! tell us which processings customers actually use.
//!
//! ## Reference planes (from the schema)
//!
//! A prismatic timber part has six global reference planes: the four longitudinal
//! faces are `1`–`4`, the two end faces `5`–`6` (user-defined planes are `100+`).
//! Every processing carries a `ReferencePlaneID` naming the plane its coordinates
//! are measured against. See [`RefPlane`].

use serde::Serialize;

/// Root `<BTLx>` document. Serialises with the design2machine default namespace
/// so the output validates against the published XSD.
#[derive(Debug, Clone, Serialize)]
#[serde(rename = "BTLx")]
pub struct Btlx {
    /// Fixed to the schema version we target.
    #[serde(rename = "@xmlns")]
    xmlns: &'static str,
    #[serde(rename = "@Version")]
    version: &'static str,
    #[serde(rename = "@Language", skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(rename = "Project")]
    pub project: Project,
}

impl Btlx {
    pub const NAMESPACE: &'static str = "https://www.design2machine.com";
    pub const VERSION: &'static str = "2.3.1";

    /// A document wrapping one project.
    pub fn new(project: Project) -> Self {
        Self {
            xmlns: Self::NAMESPACE,
            version: Self::VERSION,
            language: None,
            project,
        }
    }
}

/// `<Project>` — a named collection of parts (`ProjectType`).
#[derive(Debug, Clone, Serialize)]
pub struct Project {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "Parts", skip_serializing_if = "Option::is_none")]
    pub parts: Option<Parts>,
}

impl Project {
    pub fn new(name: impl Into<String>, parts: Vec<Part>) -> Self {
        Self {
            name: name.into(),
            parts: (!parts.is_empty()).then_some(Parts { part: parts }),
        }
    }
}

/// `<Parts>` wrapper around the repeated `<Part>` elements.
#[derive(Debug, Clone, Serialize)]
pub struct Parts {
    #[serde(rename = "Part")]
    pub part: Vec<Part>,
}

/// `<Part>` — one raw stock piece (`PartType` : `ComponentType`).
///
/// The schema requires only the stock dimensions; everything else is optional.
/// Order matters for XML: attribute fields (`@`) must precede element fields.
#[derive(Debug, Clone, Serialize)]
pub struct Part {
    #[serde(rename = "@Designation", skip_serializing_if = "Option::is_none")]
    pub designation: Option<String>,
    /// Number of identical pieces (schema default 1).
    #[serde(rename = "@Count")]
    pub count: u32,
    /// Raw stock length, mm.
    #[serde(rename = "@Length")]
    pub length: f64,
    /// Raw stock width, mm.
    #[serde(rename = "@Width")]
    pub width: f64,
    /// Raw stock height, mm.
    #[serde(rename = "@Height")]
    pub height: f64,
    #[serde(rename = "Processings", skip_serializing_if = "Option::is_none")]
    pub processings: Option<Processings>,
}

impl Part {
    /// A raw part of the given dimensions (mm), no processings yet.
    pub fn new(length: f64, width: f64, height: f64) -> Self {
        Self {
            designation: None,
            count: 1,
            length,
            width,
            height,
            processings: None,
        }
    }

    /// Set a human-readable designation (part number / label).
    pub fn designation(mut self, d: impl Into<String>) -> Self {
        self.designation = Some(d.into());
        self
    }

    /// Attach the processings to run on this part.
    pub fn with_processings(mut self, items: Vec<Processing>) -> Self {
        self.processings = (!items.is_empty()).then_some(Processings { items });
        self
    }
}

/// `<Processings>` — the ordered list of machining steps on a part.
#[derive(Debug, Clone, Serialize)]
pub struct Processings {
    /// Each item serialises as its own element (`<Drilling>`, …) via its enum tag.
    #[serde(rename = "$value")]
    pub items: Vec<Processing>,
}

/// The six global reference planes of a prismatic part (`RefSideType`), plus the
/// user-defined range. This is the `ReferencePlaneID` a processing is measured on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefPlane {
    /// Longitudinal faces 1–4, end faces 5–6.
    Global(u8),
    /// User-defined reference plane (schema: 100+).
    User(u32),
}

impl RefPlane {
    pub fn id(self) -> u32 {
        match self {
            RefPlane::Global(n) => n as u32,
            RefPlane::User(n) => n,
        }
    }
}

/// A machining step (`ProcessingType`). Externally-tagged so each variant
/// serialises as its schema element name.
#[derive(Debug, Clone, Serialize)]
pub enum Processing {
    /// `<Drilling>` — a bore on a reference plane.
    Drilling(Drilling),
    // Grow here as sample files confirm usage: JackRafterCut, Mortise, Tenon,
    // Lap, Pocket, Marking, … (see the XSD for the full list).
}

/// `<Drilling>` (`DrillingType` : `ProcessingType`). Attributes carry the common
/// processing identity; child elements carry the bore geometry.
#[derive(Debug, Clone, Serialize)]
pub struct Drilling {
    /// `Name` — processing name (required by the schema).
    #[serde(rename = "@Name")]
    name: String,
    /// `ProcessID` — unique id of this processing within the part (required).
    #[serde(rename = "@ProcessID")]
    process_id: u32,
    /// `ReferencePlaneID` — plane the coordinates are measured on (1–6 or 100+).
    #[serde(rename = "@ReferencePlaneID")]
    reference_plane_id: u32,
    /// `StartX` — position along the part, mm.
    #[serde(rename = "StartX")]
    start_x: f64,
    /// `StartY` — position across the reference plane, mm.
    #[serde(rename = "StartY")]
    start_y: f64,
    /// `Inclination` — drill angle from the plane, degrees (schema default 90 =
    /// perpendicular).
    #[serde(rename = "Inclination")]
    inclination: f64,
    /// `Depth` — bore depth, mm.
    #[serde(rename = "Depth")]
    depth: f64,
    /// `Diameter` — bore diameter, mm.
    #[serde(rename = "Diameter")]
    diameter: f64,
}

impl Drilling {
    /// A perpendicular (90°) bore at `(start_x, start_y)` on `plane`.
    pub fn new(
        name: impl Into<String>,
        process_id: u32,
        plane: RefPlane,
        start_x: f64,
        start_y: f64,
        depth: f64,
        diameter: f64,
    ) -> Self {
        Self {
            name: name.into(),
            process_id,
            reference_plane_id: plane.id(),
            start_x,
            start_y,
            inclination: 90.0,
            depth,
            diameter,
        }
    }

    /// Override the drill inclination (degrees from the reference plane).
    pub fn inclination(mut self, degrees: f64) -> Self {
        self.inclination = degrees;
        self
    }
}
