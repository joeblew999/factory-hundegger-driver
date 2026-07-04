# How This Works (and Why)

## The one idea under everything

A finished part = **a raw stick + an ordered list of cuts.** The list is the real content.

A rafter starts as a plain straight stick of timber — say 60×160mm, 4m long. The factory cuts its ends at an angle, chops a notch where it sits on the wall, drills a couple of bolt holes. Now it's a finished rafter.

The CNC machine that made it doesn't know the word "rafter." It just followed a **list of instructions**: cut this end at this angle, notch here, drill there. A BTL file *is* that list.

So: **get the list, and you can drive any machine.**

---

## Where does the list come from? (Two ways — this is the whole story)

**1. You author it.**
You tell the software "put a birdsmouth here," and it records that cut. This is how the existing timber CAD tools mostly work.

**2. Someone hands you a 3D model (an IFC file) and you work out the list.**
You have to figure out: *what cuts turn a raw stick into this shape?* This is the hard one — and it's why OpenCASCADE is in the plan. OCCT is the **geometry brain**: it takes the finished shape, subtracts the raw stick, and reads the leftover voids as cuts.

- A round void → a drill.
- A wedge void → a birdsmouth. *(That one you teach it — see below.)*

---

## The bit that trips people up: some things aren't cuts

Where the nails go. Where glue is applied. These leave **no mark** on the finished shape. You can't find them by looking at geometry, no matter how good the geometry brain is.

They only exist as **written-down data** in the file.

That's the real reason an IFC file gets read twice. It carries:

- a **shape** → send to the geometry brain, and
- **written labels/data** — nail positions, part names, which sticks group into a wall → just read as text.

Same beam, two kinds of information, two different readers. You tag everything with the beam's **ID** so the shape-derived cuts and the data-derived cuts snap back together into one list.

---

## The three design decisions (the "why")

**1. A separate geometry server.**
The geometry brain (OpenCASCADE) is heavy C++ that needs a real computer. The Cloudflare / Workers world is meant to stay light and fast. So the heavy brain sits on a proper machine, and you phone it when you need shape analysis.

**2. One list in the middle (the "op log").**
It's the single thing everyone writes to and reads from:

- the author writes to it,
- the geometry brain writes to it,
- the IFC data-reader writes to it.

Then whatever machine you feed — wood CNC (BTL) or the steel roll-former (Howick) — reads the **same list** and translates. Many ways in, many ways out, one truth in the middle. That's what keeps you from being locked to one input or one machine.

**3. Read IFC twice.**
Its two kinds of info need two different tools. Splitting them lets the light edge do the easy half (read the data) without waiting on the heavy server (analyse the shape).

---

## The sentence version

> **Shape + data → one list → any machine.**
> A heavy brain for the shape, a light layer for the data.

---

## How it maps to the stack

| Job | Where it runs | Why |
|-----|---------------|-----|
| Analyse shape → cuts (geometry brain) | Native OCCT server (heavy) | Needs a real machine; heavy C++ |
| Read IFC data (nails, names, groups) | Edge / Rust / Workers (light) | Just text parsing; stays fast |
| The op log (the one list) | The middle — synced, stored | Single source of truth |
| Emit BTL (wood) / Howick (steel) | From the op log | Same list, many outputs |
