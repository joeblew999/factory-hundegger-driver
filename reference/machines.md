# Reference machines

A sourced catalogue of the timber-CNC machines we target, and the **file format
each consumes** — which decides whether this crate's BTLx serialiser drives it
directly, or a BVX serialiser is needed. This is generic reference (public); which
*shops* run which machines is the private prospect list in
[factory-customers-cnc](https://github.com/joeblew999/factory-customers-cnc).

**Data lives in [`machines.jsonl`](machines.jsonl)** (one machine per line) — the
source of truth. The table below is **generated** from it with `mise run machines`;
edit the JSONL, not the table.

**Provenance rule:** every machine cites a `source`. A format we can't source is
`format_confirmed: false` and renders `(to confirm)` — not guessed. Format-per-model
is only partly public; see [The format picture](#the-format-picture).

## Machines

All current Hundegger machines run the **Cambium** controller, and Cambium imports
**BTL/BTLx** — so our BTLx output targets the whole current range. `BVX` is
Hundegger's own native format (saw + panel lines); needed only for a direct hand-off
that bypasses Cambium's BTLx import.

<!-- gen:machines -->

### Hundegger

| Family | Model | Controller | Interchange in | Notes |
|--------|-------|------------|----------------|-------|
| joinery | ROBOT-Max | Cambium | BTLx (via Cambium import) | 6-axis robot joinery |
| joinery | ROBOT-Drive | Cambium | BTLx (via Cambium import) | 6-axis robot + 5-axis saw/slot/marking |
| joinery | ROBOT-Compact | Cambium | BTLx (via Cambium import) | 6-axis robot + automated tool changer |
| joinery | K2-Industry | Cambium | BTLx (via Cambium import) | industrial structural-timber joinery (2026) |
| cutting | SPEED-Cut 480 | Cambium | BTLx (via Cambium import) | solid-timber cutting; native BVX (to confirm) |
| cutting | TURBO-Drive | Cambium | BTLx (via Cambium import) | flexible saw unit; native BVX (to confirm) |
| panel | SPM-2 | Cambium | BTLx (via Cambium import) | speed panel machine; also BVX via BEAVER panel interface |
| panel | PBA-Industry | Cambium | BTLx (via Cambium import) | CLT/glulam panel processor; also BVX via BEAVER |
| panel | PBA-Drive | Cambium | *(to confirm)* |  |
| panel | PBA-X | Cambium | *(to confirm)* |  |
| panel | UFA-Industry | Cambium | *(to confirm)* | CLT formatting up to 30 cm |
| panel | WALL-Master | Cambium | *(to confirm)* |  |
| planing | HM-3 | Cambium | *(to confirm)* | automatic planer; cross-section scanning |

<!-- /gen:machines -->

**Legacy naming (pre-Cambium), from [Tekla — Timber NC BVX](https://support.tekla.com/article/timber-nc-bvx):**
older docs map the **H&M line** (HM-Z / HM-D / HM-T, Trussmaster) → **BTLx** direct,
and the **SC3 / Cambium** saw → **BVX**. Those exact model names aren't in the current
lineup, so treat that mapping as historical; the modern reality is "Cambium imports
BTLx." A shop's actual model + Cambium version is the ground truth — confirm per shop.

## The format picture

- **BTLx** — the open, machine-agnostic interchange (design2machine). Every serious
  wood CAD exports it, and **Cambium imports it**, so it drives the whole current
  Hundegger range. This crate serialises BTLx. *(sources: design2machine, hsbcad
  academy Cambium notes, real 2.0.0 machine exports in `fixtures/samples`.)*
- **BVX** — Hundegger's own XML format, native to the saw (SC3/Cambium) and panel
  lines (SPM-2 / PBA / SIP via the **BEAVER** interface). Needed only for a direct,
  non-Cambium hand-off. No public samples exist. *(sources: Tekla BVX article, BEAVER
  Grasshopper interface.)*

## Adding a machine

Add a line to [`machines.jsonl`](machines.jsonl), then `mise run machines` to
regenerate the table. Fields: `manufacturer`, `model`, `family`, `controller`,
`format`, `format_via`, `format_confirmed` (bool), `source`, `notes`.

- Unknown format → `format: null`, `format_confirmed: false` (renders `(to confirm)`).
- Confirmed from a real shop file or Hundegger → set the format, `format_confirmed:
  true`, and cite it in `source` (e.g. `"shop export 2026-07"`).
- Other manufacturers (Weinmann, Krüsi, Essetre…) → just add rows with their
  `manufacturer`; they mostly take **BTLx** too, which is the point.
