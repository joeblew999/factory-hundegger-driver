# factory-hundegger-driver

The **Hundegger** timber-CNC machine driver for the [factory-floor](https://github.com/joeblew999/factory-floor)
stack — sibling to [factory-howick-driver](https://github.com/joeblew999/factory-howick-driver)
(cold-formed steel). Two parts:

- **`btlx`** — a serialiser for [BTLx](https://www.design2machine.com/), the open,
  machine-agnostic interchange format for timber fabrication. Turn a parametric
  part description into a `.btlx` file that validates against the published schema.
  This is the reusable, machine-independent core.
- **`driver`** — `Hundegger`, which implements the
  [`factory-machine-model`](https://github.com/joeblew999/factory-machine-model)
  `MachineDriver` contract: it hands a BTLx payload to the machine's controller and
  reports state, so the gateway exposes a standard OPC-UA address space.

```rust
use factory_hundegger_driver::btlx::{model::*, to_xml};

let part = Part::new(3000.0, 160.0, 80.0)          // 3 m beam, 160×80 mm
    .designation("beam-1")
    .with_processings(vec![Processing::Drilling(
        Drilling::new("bolt-hole", 1, RefPlane::Global(3), 500.0, 80.0, 80.0, 12.0),
    )]);
let xml = to_xml(&Btlx::new(Project::new("demo", vec![part])))?;   // valid BTLx 2.3.1
```

## Why BTLx first

The timber-CNC market has already standardised the hard part. **BTLx is the
universal interchange** — every serious wood CAD exports it — and several
commercial post-processors already turn BTLx into machine NC-code (Hundegger's own
**Cambium**, **NC-HOPS** by direkt cnc-systeme, AGACAD's Revit→BVX exporter, Tekla).
So we don't reinvent the machine post-processor. Our leverage is the two ends the
incumbents don't own for our customers: **design → BTLx** generation, and the
factory-floor **orchestration + telemetry** around whatever controller the shop runs.

BVX (Hundegger's own format, also XML; used by the panel line SPM-2/PBA/SIP and the
SC3/Cambium saw) is a second serialiser we add only when a specific machine needs it.

The background research and the customer/market context live in
[factory-customers-cnc/customers/austria-cnc](https://github.com/joeblew999/factory-customers-cnc).

## Status

Early scaffold, but the core is proven end-to-end:

- Typed Rust model of the BTLx **document → project → part → processings** structure.
- **`Drilling`** as the first concrete processing (the [`Processing`] enum grows as
  real sample files tell us which processings customers actually use).
- Output **validates against the real BTLx 2.3.1 XSD** — see
  [`fixtures/`](fixtures/) and run `xmllint` yourself.
- `Hundegger` implements the full `MachineDriver` contract, dispatching a BTLx
  payload to the machine.

## Open questions — need a real shop or Hundegger

Desk research can't close these; a single real job bundle would close most:

- **Ingest mechanism.** How Cambium actually takes a file — watched hot folder,
  manual import, or an API. `run_job` writes a valid file to the dispatch dir as the
  best-known hand-off; swap in the real path once known.
- **Telemetry format.** The driver reports only a dispatch counter. Real machine
  state / job feedback needs a sample of the controller's status-log format.
- **Which processings.** We have `Drilling`; real files show which of the 40+ BTLx
  processings (cuts, mortises, tenons, laps, pockets…) customers use, and in what mix.
- **BTLx vs BVX.** Which format a given customer machine needs.

**The one artifact that unblocks the most:** a real `.btlx`/`.bvx` export from a shop,
ideally with that machine's output log. See [factory-customers-cnc](https://github.com/joeblew999/factory-customers-cnc)
for the customer-facing ask.

## Develop

```sh
cargo test                              # unit + doctests
cargo run --example emit                # print a sample BTLx to stdout
xmllint --noout --schema fixtures/schema/BTLx_2_3_1.offline.xsd fixtures/sample-drilling.btlx
```

License: MIT OR Apache-2.0.
