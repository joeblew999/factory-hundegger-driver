# Reference machines

A sourced catalogue of the timber-CNC machines we target, and the **file format
each consumes** — which decides whether this crate's BTLx serialiser drives it
directly, or a BVX serialiser is needed. This is generic reference (public); which
*shops* run which machines is the private prospect list in
[factory-customers-cnc](https://github.com/joeblew999/factory-customers-cnc).

**Provenance rule:** every row traces to a source. A claim we can't source is marked
`(to confirm)` — not guessed. Format-per-model in particular is only partly public;
see [The format picture](#the-format-picture).

## Hundegger

The current lineup, from [hundegger.com/en/machines](https://www.hundegger.com/en/machines)
(2026). All current machines run Hundegger's **Cambium** controller software, and
Cambium imports **BTL/BTLx** — so our BTLx output targets the whole current range.
`BVX` is Hundegger's own native format (saw + panel lines); needed only if driving a
machine *without* going through Cambium's BTLx import.

| Family | Model | Controller | Interchange in | Source |
|--------|-------|------------|----------------|--------|
| Joinery / robot | ROBOT-Max | Cambium | BTLx (via Cambium) | hundegger.com |
| Joinery / robot | ROBOT-Drive | Cambium | BTLx (via Cambium) | hundegger.com |
| Joinery / robot | ROBOT-Compact | Cambium | BTLx (via Cambium) | hundegger.com |
| Joinery | K2-Industry (2026) | Cambium | BTLx (via Cambium) | hundegger.com |
| Cutting | SPEED-Cut 480 | Cambium | BTLx (via Cambium); native BVX *(to confirm)* | hundegger.com |
| Cutting | TURBO-Drive | Cambium | BTLx (via Cambium); native BVX *(to confirm)* | hundegger.com |
| Panel | SPM-2 | Cambium | BVX via the **BEAVER** panel interface; BTLx via Cambium *(to confirm)* | hundegger.com · Tekla/BEAVER |
| Panel | PBA-Industry | Cambium | BVX via **BEAVER**; BTLx via Cambium *(to confirm)* | hundegger.com · BEAVER |
| Panel | PBA-Drive | Cambium | *(to confirm)* | hundegger.com |
| Panel | PBA-X | Cambium | *(to confirm)* | hundegger.com |
| Panel | UFA-Industry | Cambium | *(to confirm — CLT formatting up to 30 cm)* | hundegger.com |
| Panel | WALL-Master | Cambium | *(to confirm)* | hundegger.com |
| Planing | HM-3 | Cambium | *(to confirm — automatic planer, cross-section scan)* | hundegger.com |

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

## Adding rows

- New Hundegger model → add to the table with a source, format `(to confirm)` until
  we can back it.
- Other manufacturers (Weinmann, Krüsi, Essetre…) → new `## <manufacturer>` section.
  They mostly consume **BTLx** too (it's the open standard), which is the point.
- Confirmed a format from a real shop file or Hundegger? Drop the `(to confirm)` and
  cite the source (e.g. "shop export, 2026-07").
