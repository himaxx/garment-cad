# Implementation Plan

**Strategy:** build in the order that delivers value early and puts the biggest risk (nesting quality)
first. The order is the reverse of how the product is used:

```
Plot & files  ──►  Marker / nesting  ──►  Pattern editor  ──►  Grading  ──►  Advanced design
 (useful in       (go/no-go gate:        (replaces DGS        (completes      (darts, pleats,
  month 1)         beat Richpeace GMS)    basics)              DGS core)       slash & spread)
```

Richpeace DGS keeps running in production until our pattern editor is ready, so production is never at risk.

**Assumed team:** 2 developers (1 Rust/core, 1 UI) + pattern master (part-time, about 30%) + marker maker (testing).
With 1 developer, roughly double the durations.

---

## Phase 0 — Foundation & technical spikes (weeks 1–3)

| # | Task | Output |
|---|---|---|
| 0.1 | Repo, Cargo workspace, Tauri app shell, CI (Windows + Linux) | Empty app builds & runs |
| 0.2 | **Collect real data** from the factory (see checklist below) | `testdata/` folder |
| 0.3 | Spike S1: run jagua-rs/sparrow on 3 real markers; compare efficiency vs Richpeace GMS; check licences | Go/adjust decision on nesting engine |
| 0.4 | Spike S2: seam-allowance offset with Clipper2 on curved pieces, all corner types | Choose geometry library |
| 0.5 | Spike S3: send a hand-written HPGL test square to our Richpeace plotter over TCP | Confirms port/protocol |
| 0.6 | Spike S4: parse a Richpeace DXF-AAMA export | Confirms layer usage |

**Data checklist from the factory**
- [ ] 10+ styles exported from Richpeace DGS as **DXF-AAMA + RUL** (shirt, trouser, kurta, T-shirt…)
- [ ] The same styles' **marker files + PLT** from GMS, with **length & efficiency noted**
- [ ] Plotter model, IP address, paper width; photo of plotter network settings
- [ ] 5 DXF files received from buyers (Gerber/Lectra/Optitex exports, if any)
- [ ] List of the 30 tools the pattern master uses most (watch them work for 1–2 days)

**Exit:** all 4 spikes pass; nesting engine chosen.

---

## Phase 1 — Core geometry, file I/O, plotting (weeks 4–9) → Milestone M1

Features: `PD-01..03, PD-220, GR-13, PL-01, PL-02, PL-04..07, PL-13, ST-01`

| # | Task |
|---|---|
| 1.1 | `geom` crate: Vec2, Bézier, flattening, arc length, point-at-distance, intersections, offsets |
| 1.2 | `model` crate: Style, Piece, Vertex, internals, size range; JSON save/load with version |
| 1.3 | `io`: DXF-AAMA/ASTM + RUL **import** (pieces, notches, drills, grain, text, grades) |
| 1.4 | `io`: HPGL/PLT reader & writer |
| 1.5 | `plot`: plotter profiles, calibration, TCP driver, persistent queue |
| 1.6 | UI: canvas viewer (zoom/pan/1:1), piece list, size selector (graded sizes from RUL), plot queue screen |
| 1.7 | CLI: `gcad convert`, `gcad plot` |

**M1 acceptance:** open any Richpeace DXF export, see all pieces and sizes correctly, plot patterns at 1:1
on our plotter within ±1 mm/m, and plot external PLT files (PlotCenter replaced).

---

## Phase 2 — Marker system (weeks 10–19) → Milestone M2 + **GO/NO-GO gate**

Features: `MK-01..05, MK-20, MK-21, MK-23, MK-24, MK-40..45, MK-60, MK-61, MK-63, MK-80..82, RP-03`

| # | Task |
|---|---|
| 2.1 | Marker order dialog: style, fabric type, size ratio, width, margins, shrinkage, plies |
| 2.2 | Piece preparation: cut-line polygons per size, quantity × mirror pairs, buffers, orientation rules |
| 2.3 | `marker` crate: placement model, collision check, **bump** (arrow keys), rotate/flip, overlap highlight |
| 2.4 | `nest` crate: adapter to the nesting engine, background worker, time limit, best-so-far updates to UI |
| 2.5 | Interactive: lock placed pieces + auto-fill |
| 2.6 | Marker UI: unplaced bin, live length/efficiency/piece counter |
| 2.7 | Marker report PDF; marker → PLT → plotter |
| 2.8 | Benchmark suite: 20 real markers, automated efficiency comparison vs Richpeace |

**M2 acceptance:** marker maker produces real production markers end to end (import → nest → plot) without Richpeace.

### 🚦 Go/no-go gate (end of week 19)
- Our efficiency **≥ Richpeace GMS − 0.5 %** on average over 20 benchmark markers → **continue** to Phase 3.
- Else → spend at most 4 more weeks tuning; if still behind, **stop or pivot** (keep plot/marker tools,
  buy a commercial CAD). The money spent so far is limited and the plotting tools are still useful.

---

## Phase 3 — Pattern editor MVP (weeks 20–35) → Milestone M3

Features: all **M** items in B1–B5, B8–B10, `ST-02..05, ST-09, RP-01, RP-04`

| # | Task |
|---|---|
| 3.1 | Command system: undo/redo, selection, properties panel, numeric input bar |
| 3.2 | Snapping engine & tool framework (each tool = state machine) |
| 3.3 | Drafting tools: point, line, curve, rectangle, parallel, perpendicular, trim/extend/split/join, move/copy/rotate/mirror, divide |
| 3.4 | Measuring tools: tape, compare lengths, angle, area |
| 3.5 | Piece creation from draft outline; piece properties; fold/unfold; split piece; copy |
| 3.6 | Point/curve editing, lengthen/shorten, shrinkage |
| 3.7 | Seam allowance with all corner types (auto-update) |
| 3.8 | Notches (all types, positioning modes), drills, buttons, internal lines, grain, labels |
| 3.9 | DXF-AAMA **export** + round-trip tests with Richpeace/Gemini |
| 3.10 | Basic darts (`PD-100`) |

**M3 acceptance:** pattern master drafts a basic shirt and trouser from scratch, adds seam allowance and notches,
exports DXF that opens correctly in Richpeace, and the marker module uses it directly.

---

## Phase 4 — Grading (weeks 36–43) → Milestone M4

Features: `GR-01..09, GR-12`

| # | Task |
|---|---|
| 4.1 | Size range & break sizes editor |
| 4.2 | Grade rule entry per point (incremental/cumulative), rule table library |
| 4.3 | Copy/paste/flip grades; distribute along curve |
| 4.4 | Stack view with alignment options; extract single size |
| 4.5 | Graded measurement report (Excel) |
| 4.6 | Internals & seam allowance follow grading — full regression tests |

**M4 acceptance:** a full style is designed, graded, nested and plotted **entirely in our software**.
→ Start a **parallel run** with Richpeace on real orders for 4 weeks.

---

## Phase 5 — Advanced design tools (weeks 44–53)

Features: `PD-27, PD-42, PD-44, PD-101..104, PD-120..122, PD-142, PD-164, PD-200..202, PD-221, PD-224, MK-22, MK-25, MK-46, MK-47, PL-09..11, ST-06..08`

Order by what the pattern master actually asks for during the parallel run.

---

## Phase 6 — Rollout & hardening (weeks 54–60)

- Windows installer, auto-update, crash reporting
- Training material (short videos in Hindi/English), keyboard-shortcut cheat sheet
- Migrate style library from Richpeace (batch DXF import)
- Switch off Richpeace for new styles

## Phase 7+ — Growth (after go-live)

Formula/parametric drafting (PD-29), digitizer (PD-222), photo digitizing (PD-223), stripe/check matching (MK-26),
cut order planning (MK-83), costing & ERP integration (RP-06/07), batch nesting (MK-62), multi-user library (ST-10),
Hindi UI (SY-02), and licensing if we decide to sell (SY-04).

---

## Timeline summary (2 developers)

| Phase | Weeks | Cumulative | Milestone |
|---|---|---|---|
| 0 Foundation & spikes | 3 | month 1 | Tech decisions made |
| 1 Files & plotting | 6 | month 2 | **M1** – PlotCenter replaced |
| 2 Marker | 10 | month 5 | **M2** – GMS replaced, **🚦 gate** |
| 3 Pattern editor | 16 | month 8–9 | **M3** – basic DGS replaced |
| 4 Grading | 8 | month 10–11 | **M4** – full workflow, parallel run |
| 5 Advanced tools | 10 | month 13 | Feature parity for daily work |
| 6 Rollout | 6 | month 14–15 | Richpeace retired for new styles |

## Main risks

| Risk | Mitigation |
|---|---|
| Nesting worse than GMS (costs fabric every day) | Phase 2 early + benchmark + go/no-go gate |
| Pattern masters resist change | Involve from week 1; copy familiar shortcuts; numeric input everywhere; parallel run |
| DXF incompatibility with buyer/Richpeace files | Real sample files as tests from day 1 |
| Scope creep (DGS has 200+ tools) | Only build tools from the "top 30" list + parallel-run requests |
| Developer leaves | Docs, tests, code reviews, two people know each area |
| Open-source licence conflicts | Licence review in Phase 0 before depending on any library |

## Definition of done (every feature)

1. Works for **all sizes** (graded), not just base
2. Undo/redo works
3. Saved & reopened correctly
4. Unit tests (+ golden file test if it touches import/export/plot)
5. Checked by pattern master or marker maker on a real style
