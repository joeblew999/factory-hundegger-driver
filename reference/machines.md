# Reference machines

A sourced catalogue of the timber-CNC machines we target, and the **file format
each consumes** — which decides whether this crate's BTLx serialiser drives it
directly, or a BVX serialiser is needed. This is generic reference (public); which
*shops* run which machines is the private prospect list in
[factory-customers-cnc](https://github.com/joeblew999/factory-customers-cnc).

**Data lives in JSONL** — the source of truth: [`makers.jsonl`](makers.jsonl)
(manufacturer level) and [`machines.jsonl`](machines.jsonl) (Hundegger model level).
Both tables below are **generated** with `mise run machines`; edit the JSONL, not the
tables.

**Provenance rule:** every row cites a `source`. A format we can't source renders
`(to confirm)` — not guessed.

## Makers — where to steer effort

Every maker and the format they take, ordered by market position. **`popularity` is a
qualitative tier** (leader / major / established / niche) drawn from company
positioning statements and product breadth — **not sales data.** It's a rough steer,
not a fact.

<!-- gen:makers -->

| Manufacturer | Country | Segment | BTL | BTLx | Native | Drive today? | Popularity | Source |
|---|---|---|:---:|:----:|---|---|---|---|
| Hundegger | DE | joinery | ✓ | ✓ | BVX2 | **yes** (BTLx) | leader | hundegger.com; cadwork KB; Tekla |
| Weinmann (HOMAG Group) | DE | framing | ✓ | ✓ | — | **yes** (BTLx) | leader | homag.com; ansvarcad; AGACAD |
| Essetre | IT | mass-timber | ✓ | ? | — | ? | major | timbertools.com; essetre-na.com; ansvarcad |
| Krüsi (Krusimatic) | CH | joinery | ✓ | ? | — | ? | established | krusi.com; timbertools.com |
| Randek | SE | framing | ? | ? | CDT4 / SPL728 | ? | major | agacad.com; archiframe.fi |
| SCM | IT | multi | ✓ | ? | — | ? | major | ansvarcad |
| Schmidler | DE | joinery | ✓ | ? | — | ? | niche | ansvarcad |
| Baljer & Zembrod | DE | mass-timber | ? | ? | — | ? | niche | cadwork.com |
| CMS | IT | multi | ? | ? | — | ? | unknown | cadwork.com |
| Stromab | IT | cutting | ? | ? | — | ? | niche | archiframe.fi |
| Creneau Industriel | BE | framing | ? | ? | — | ? | unknown | cadwork.com |
<!-- /gen:makers -->

**The steer** — the market splits into three buckets (reproducible from the JSONL):

1. **Drive today with our BTLx** → **Hundegger** and **Weinmann** — the two leaders,
   in *different* segments (joinery/beam vs frame/prefab). BTLx already reaches both.
2. **BTL-only, needs a BTL v10 serialiser** → **Essetre, Krüsi, SCM, Schmidler**
   (major/established). One extra serialiser opens this whole tier.
3. **Proprietary, needs its own exporter** → **Randek** (major, framing) uses **CDT4 /
   SPL728**, not BTL/BTLx at all — a separate, bespoke effort, only if a Randek
   customer appears.

Remaining `?` rows (Baljer & Zembrod, CMS, Stromab, Creneau) are unverified — not
worth chasing until one is a real prospect. Effort priority: BTLx (**done**) → BTL v10
(opens tier 2) → Randek's format (on demand only).

## Hundegger machines

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
| cutting | SPEED-Cut 480 | Cambium | BTLx (via Cambium import) | solid-timber cutting; BVX2 native |
| cutting | TURBO-Drive | Cambium | BTLx (via Cambium import) | flexible saw unit; BVX2 native |
| panel | SPM-2 | Cambium | BTLx (via Cambium import) | speed panel machine; also BVX via BEAVER panel interface |
| panel | PBA-Industry | Cambium | BTLx (via Cambium import) | CLT/glulam panel processor; also BVX via BEAVER |
| panel | PBA-Drive | Cambium | BTLx (via Cambium import) | panel processing |
| panel | PBA-X | Cambium | BTLx (via Cambium import) | panel processing |
| panel | UFA-Industry | Cambium | BTLx (via Cambium import) | CLT formatting up to 30 cm |
| panel | WALL-Master | Cambium | BTLx (via Cambium import) | wall prefab panel machine |
| planing | HM-3 | Cambium | BTLx (via Cambium import) | automatic planer; cross-section scanning |

<!-- /gen:machines -->

**Legacy naming (pre-Cambium), from [Tekla — Timber NC BVX](https://support.tekla.com/article/timber-nc-bvx):**
older docs map the **H&M line** (HM-Z / HM-D / HM-T, Trussmaster) → **BTLx** direct,
and the **SC3 / Cambium** saw → **BVX**. Those exact model names aren't in the current
lineup, so treat that mapping as historical; the modern reality is "Cambium imports
BTLx." A shop's actual model + Cambium version is the ground truth — confirm per shop.

## The format picture — do other makers use the same protocol?

Partly. There are three formats, and the honest cross-vendor reality (verified):

- **BTL (v10)** — the **older** but most widely-supported open standard.
  **Weinmann, SCM, Essetre, Krüsi and Schmidler** all read BTL10, and the `.btl`
  extension is used worldwide across many makers. *(source:
  [ansvarcad CNC files](https://www.ansvarcad.com/features/cnc-files/).)*
- **BTLx** — the **newer**, XML, machine-independent variant of BTL
  (design2machine) — **what this crate serialises.** Hundegger/Cambium imports it and
  Weinmann reads it, but the sources call BTLx **"not yet widespread"** — BTL's reach
  is broader. So **do not assume a given maker takes BTLx**; confirm per maker/model.
  *(sources: [design2machine](https://www.design2machine.com/); ansvarcad;
  [AGACAD Weinmann BTL/BTLx exporter](https://agacad.com/products/bim-solutions/wood-framing-cnc-exporter-weinmann-btl/overview).)*
- **BVX** — Hundegger's own native format (saw + panel lines, via the **BEAVER**
  interface). Hundegger-specific, no public samples. *(sources: Tekla BVX; BEAVER.)*

**Makers seen in the BTL/BTLx post-processor ecosystem** (manufacturer level, per
cadwork / ArchiFrame post-processor lists): Hundegger, Weinmann, Baljer & Zembrod,
Krüsi / Krusimatic / Lignamatik, Essetre, Randek, CMS, Creneau Industriel, SCM,
Schmidler, Stromab. Which *dialect* (BTL vs BTLx) and which *models* is not verified
here — add to `machines.jsonl` only with a source.

**Product implication:** we emit BTLx, which is exactly right for Hundegger/Cambium
(our first target). To reach the broader BTL-reading market later, we may need to
also emit the older **BTL v10** — a separate serialiser, tracked when a real
non-Hundegger customer needs it.

## Adding rows

Edit the JSONL, then `mise run machines` to regenerate both tables. Every row needs a
`source`; unknowns stay `(to confirm)` — don't guess.

- **A maker** → [`makers.jsonl`](makers.jsonl). Queryable fields: `manufacturer`,
  `group`, `country`, `segment` (joinery/framing/panel/cutting/mass-timber/multi),
  `reads_btl` / `reads_btlx` / `reads_bvx` (**`true` / `false` / `null`=unverified** —
  this is what makes "who can we drive?" answerable), `popularity`
  (leader/major/established/niche/unknown), `confidence` (high/medium/low), `source`,
  `native_format` (the maker's own machine format, e.g. `BVX2`, `CDT4 / SPL728`, or
  `null`), `notes`. Never guess a format to `true` — leave it `null` until sourced. Example
  query: `open makers.jsonl | lines | each {from json} | where reads_btlx == true`.
- **A Hundegger model** → [`machines.jsonl`](machines.jsonl): `manufacturer`, `model`,
  `family`, `controller`, `format`, `format_via`, `format_confirmed` (bool), `source`,
  `notes`. Confirmed from a real shop file → set the format and cite it (e.g.
  `"shop export 2026-07"`).
