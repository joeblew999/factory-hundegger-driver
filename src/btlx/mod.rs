//! BTLx — the open timber-fabrication interchange format we serialise to.
//!
//! [`model`] holds the schema-mirrored data types; [`to_xml`] renders a
//! [`Btlx`](model::Btlx) document to a machine-agnostic `.btlx` string that
//! validates against `fixtures/schema/BTLx_2_3_1.xsd`.
//!
//! ```
//! use factory_hundegger_driver::btlx::{model::*, to_xml};
//!
//! let part = Part::new(3000.0, 160.0, 80.0)
//!     .designation("beam-1")
//!     .with_processings(vec![Processing::Drilling(
//!         Drilling::new("bolt-hole", 1, RefPlane::Global(3), 500.0, 80.0, 80.0, 12.0),
//!     )]);
//! let doc = Btlx::new(Project::new("demo", vec![part]));
//! let xml = to_xml(&doc).unwrap();
//! assert!(xml.contains("<Drilling"));
//! ```

pub mod model;

pub use model::{Btlx, Drilling, Part, Parts, Processing, Processings, Project, RefPlane};

/// Render a BTLx document to an indented XML string, with the standard
/// declaration header.
///
/// Output uses the design2machine default namespace and no element prefixes, as
/// required by the schema's `elementFormDefault="qualified"`.
pub fn to_xml(doc: &Btlx) -> Result<String, quick_xml::se::SeError> {
    use serde::Serialize;
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    let mut ser = quick_xml::se::Serializer::new(&mut out);
    ser.indent(' ', 2);
    doc.serialize(ser)?;
    out.push('\n');
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::model::*;
    use super::to_xml;

    fn sample_doc() -> Btlx {
        let part = Part::new(3000.0, 160.0, 80.0)
            .designation("beam-1")
            .with_processings(vec![
                Processing::Drilling(Drilling::new(
                    "bolt-hole-1",
                    1,
                    RefPlane::Global(3),
                    500.0,
                    80.0,
                    80.0,
                    12.0,
                )),
                Processing::Drilling(
                    Drilling::new("angled", 2, RefPlane::Global(1), 1500.0, 40.0, 60.0, 10.0)
                        .inclination(45.0),
                ),
            ]);
        Btlx::new(Project::new("demo-project", vec![part]))
    }

    #[test]
    fn emits_well_formed_btlx() {
        let xml = to_xml(&sample_doc()).unwrap();
        // Root, namespace, version.
        assert!(xml.contains("<BTLx"));
        assert!(xml.contains("xmlns=\"https://www.design2machine.com\""));
        assert!(xml.contains("Version=\"2.3.1\""));
        // Project + part dimensions as attributes.
        assert!(xml.contains("<Project Name=\"demo-project\""));
        assert!(xml.contains("Length=\"3000\""));
        // Two drillings, geometry as child elements.
        assert_eq!(xml.matches("<Drilling").count(), 2);
        assert!(xml.contains("<Diameter>12</Diameter>"));
        assert!(xml.contains("<Inclination>45</Inclination>"));
        // ReferencePlaneID carried as attribute.
        assert!(xml.contains("ReferencePlaneID=\"3\""));
    }
}
