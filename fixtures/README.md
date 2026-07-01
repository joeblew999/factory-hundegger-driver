# fixtures

Generic, non-customer BTLx artifacts — the format examples and the schema we
validate against. (Anything tied to a real customer or project lives in the
private [factory-customers-cnc](https://github.com/joeblew999/factory-customers-cnc),
not here.)

| File | What |
|------|------|
| `schema/BTLx_2_3_1.xsd` | The canonical BTLx 2.3.1 schema, verbatim from [design2machine](https://www.design2machine.com/btlx/schema.html). |
| `schema/BTLx_2_3_1.offline.xsd` | The same schema with the one external `<xs:include>` (the X3D shape-geometry extension, fetched over the network) and the unused `ShapeType` removed, so it compiles and validates **offline**. We never emit `<Shape>`, so this validates the exact subset we produce. |
| `sample-drilling.btlx` | Output of `cargo run --example emit` — a 3 m beam with two drillings. Regenerate it, don't hand-edit. |

## Validate our output against the schema

```sh
cargo run --example emit > fixtures/sample-drilling.btlx
xmllint --noout --schema fixtures/schema/BTLx_2_3_1.offline.xsd fixtures/sample-drilling.btlx
# -> fixtures/sample-drilling.btlx validates
```

The canonical `BTLx_2_3_1.xsd` will *fail* to compile under xmllint offline
because it pulls `x3d-3.3.xsd` from web3d.org; use the `.offline.xsd` for local /
CI validation.

## Wanted: a real customer sample

The single most useful fixture we don't have yet is a **real `.btlx` (or `.bvx`)
export from an actual shop** — a known-good file for a common joint, ideally paired
with the machine's output/status log. That one artifact validates our serialiser
against reality and unblocks the dispatch + telemetry work. See the repo README's
open questions.
