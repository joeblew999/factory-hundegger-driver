# Real-world BTLx samples

Actual `.btlx` files found in public repos, vendored here (with provenance) so we
can test our BTLx handling against reality — and against real machine output, not
hand-written toys. **BVX samples are not in here** — a public search turned up zero
real Hundegger `.bvx` files (the `.bvx` extension collides with a meteorology
format), confirming BVX has to come from a shop.

| File | Source | Version | Notes |
|------|--------|---------|-------|
| `eth-stencil_60x80.btlx` | [gramaziokohler/coding_architecture_fs26_focus_work](https://github.com/gramaziokohler/coding_architecture_fs26_focus_work) | 2.0.0 | real machine export, 38 parts, Lap/JackRafterCut/Drilling |
| `easyhops-test.btlx` | [MAS-dfab/easyhops](https://github.com/MAS-dfab/easyhops) | 2.0.0 | real export |
| `lignocam-hackathon.btlx` | [markoczy/AECHackaton_ProjectsTalking](https://github.com/markoczy/AECHackaton_ProjectsTalking) | 2.2.0 | small real export |
| `btlx-parser-sample.btlx` | [victorwhale/btlx-parser](https://github.com/victorwhale/btlx-parser) | 2.0 | ⚠️ **non-conformant toy** — puts geometry in attributes, not child elements. Kept as a counter-example; do **not** model against it. |

These are third-party files retained for interoperability testing, each attributed
to its source above (fetch a fresh copy from the linked repo if ever needed).

## What these files taught us

1. **Real files are BTLx 2.0.0 / 2.2.0 — not 2.3.1.** Our first schema (`BTLx_2_3_1.xsd`)
   is the latest, but real exporters in the wild emit older versions. The 2.0.0
   schema is now committed alongside it. The processing types we care about have the
   **same structure** across 2.0.0→2.3.1; the version differences are elsewhere
   (e.g. the `Shape` geometry block, element ordering). Which version a given
   customer's Cambium wants is still a shop question.

2. **The real processing mix** (across a 400+-processing sample set):
   `Lap` ≫ `JackRafterCut` > `Drilling`, with `Mortise`/`Tenon` rare in these
   panel/stencil jobs. So the processings to implement first are **Lap and
   JackRafterCut**, then Drilling — not Drilling alone.

3. **Geometry is child elements, not attributes.** Real machine files write
   `<JackRafterCut ...><StartX>90.000</StartX>...</JackRafterCut>` — the common
   processing identity (`Name`, `ProcessID`, `ReferencePlaneID`) as attributes, the
   geometry as child elements. This matches the XSD and our model. (A popular
   GitHub "sample" that uses attributes instead is a non-conformant toy; ignore it.)

4. **Conformance:** the Parts and Processings in the real files validate against the
   version-matched schema. The only bits that don't are the optional `<Shape>` X3D
   geometry blocks, which need the external web3d schema and which we don't emit
   anyway (see [`../README.md`](../README.md) for the offline-validation note).
