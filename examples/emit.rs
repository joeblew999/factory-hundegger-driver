//! Emit a sample BTLx document to stdout.
//!
//! ```sh
//! cargo run --example emit > fixtures/sample-drilling.btlx
//! xmllint --noout --schema fixtures/schema/BTLx_2_3_1.xsd fixtures/sample-drilling.btlx
//! ```

use factory_hundegger_driver::btlx::{model::*, to_xml};

fn main() -> anyhow::Result<()> {
    // A 3 m beam, 160×80 mm, with two bolt holes on different reference planes.
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
                Drilling::new(
                    "bolt-hole-2",
                    2,
                    RefPlane::Global(1),
                    2500.0,
                    80.0,
                    80.0,
                    12.0,
                )
                .inclination(60.0),
            ),
        ]);

    let doc = Btlx::new(Project::new("sample-project", vec![part]));
    print!("{}", to_xml(&doc)?);
    Ok(())
}
