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

## For factory partners — check it against your own files

You do **not** need to be a programmer or install anything to help us validate this.

1. Go to the [**Releases**](https://github.com/joeblew999/factory-hundegger-driver/releases)
   page and download the `hundegger-btlx` file for your system
   (Windows / macOS / Linux).
2. Open a terminal (on Windows: PowerShell) in the folder where it downloaded, and
   run it on a `.btlx` file your CAD or machine produced:

   ```
   hundegger-btlx inspect my-real-file.btlx
   ```

   It prints the BTLx version, how many parts it found, and every processing type in
   the file — for example:

   ```
   Version: 2.0.0
   Parts:   38
   Processings (130 total):
        20  Drilling               [ok]
        46  JackRafterCut          [read-only]
        64  Lap                    [read-only]

   We can READ this file. We cannot yet WRITE these processing types: JackRafterCut, Lap.
   ```

3. **Send us the output** (and the file if you can). It tells us exactly which
   processings your shop actually uses, so we build those first — this is how we
   turn "it should work" into "we ran it on your real jobs and it does."

That's the validation loop: your real files drive what we build, and the tool proves
we read them correctly before anything ever reaches a machine. `hundegger-btlx demo`
prints a sample BTLx file so you can see what we generate.

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

Working, and proven against real machine files — not just a scaffold:

- **`hundegger-btlx inspect`** reads any real `.btlx` and reports version, parts, and
  the processing histogram. Tested on real 2.0.0 machine exports (see
  [`fixtures/samples`](fixtures/samples/SAMPLES.md)).
- **Writing:** typed model of the BTLx **document → project → part → processings**;
  `Drilling` implemented, output **validates against the real BTLx XSD** (`xmllint`).
  Real-file analysis says **`Lap` and `JackRafterCut` are next** (by far the most
  common processings in the wild).
- `Hundegger` implements the full `factory-machine-model` `MachineDriver` contract.
- Real files are BTLx **2.0.0 / 2.2.0** in practice; both the 2.0.0 and 2.3.1 schemas
  are in [`fixtures/schema`](fixtures/schema).

## Open questions — need a real shop or Hundegger

The `inspect` tool is designed to help close these from the field:

- **Which processings.** `inspect` on a shop's own files answers this directly. First
  real reports say Lap ≫ JackRafterCut > Drilling.
- **Ingest mechanism.** How Cambium takes a file — watched hot folder, manual import,
  or an API. `run_job` writes a valid file to the dispatch dir as the best-known
  hand-off; swap in the real path once known.
- **Telemetry format.** The driver reports only a dispatch counter; real machine
  feedback needs a sample of the controller's status-log format.
- **BTLx vs BVX, and which version.** Which format/version a given customer machine
  wants. (No public BVX samples exist — must come from a shop.)

## Develop

Quick loop with plain cargo:

```sh
cargo test                                        # unit + doctests
cargo run --bin hundegger-btlx -- inspect fixtures/samples/eth-stencil_60x80.btlx
cargo run --bin hundegger-btlx -- demo            # print a sample BTLx
xmllint --noout --schema fixtures/schema/BTLx_2_3_1.offline.xsd fixtures/sample-drilling.btlx
```

Rust is pinned in [`rust-toolchain.toml`](rust-toolchain.toml) (rustup) — **not**
mise. `xmllint` is only needed for the schema-validation check.

## CI & releasing — how binaries reach the factory

CI is a single mise task, `mise run ci`, that runs **identically on your machine and
on the GitHub matrix** (ubuntu + macOS + windows). This repo consumes the shared
[joeblew999/.github](https://github.com/joeblew999/.github) task library by reference
(pinned `?ref=` in [`mise.toml`](mise.toml)); the workflows under
[`.github/workflows`](.github/workflows) are **generated** by
`mise run mise:repo:bootstrap` — don't hand-edit them.

```sh
mise run mise:global:bootstrap    # once per machine — seeds nu, gh, git-cliff…
mise run ci                       # the whole of CI: bootstrap-check + rust:test + smoke
```

What `mise run ci` runs (the `[tasks.ci].depends` list in `mise.toml`):

- **`rust:test`** — `cargo test --all-targets`, so the matrix compiles the CLI on
  every OS (native, no cross-compilation).
- **`smoke`** — runs the built tool on a real machine `.btlx`, on every OS.
- **`bootstrap-check`** — fails if the generated workflows drift from the shared
  bootstrap.

**Releasing binaries** (what puts the `.exe` in Max's hands): publishing is gated on
`CI_RELEASE = true` in `mise.toml` and runs on a **git tag**, not every push. Each OS
runner builds its own arch via the `dist` task and attaches the archive to the
GitHub Release. **Windows is in the release matrix on purpose** — Cambium is Windows.

```sh
mise run release:github -- v0.1.0      # changelog → tag → GitHub release → binaries build
# or just:  git tag v0.1.0 && git push origin v0.1.0
```

Downloadable binaries land on the
[Releases](https://github.com/joeblew999/factory-hundegger-driver/releases) page for
Windows / macOS / Linux. `mise run run-bin` fetches and runs the published binary for
your OS.

License: MIT OR Apache-2.0.
