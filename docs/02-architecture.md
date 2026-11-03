# Architecture & Technical Design

## 1. Technology choices

| Layer | Choice | Why |
|---|---|---|
| Core engine | **Rust** (Cargo workspace) | Fast, memory-safe, great for geometry & nesting; compiles to native and to WebAssembly (future web/cloud version) |
| Desktop shell | **Tauri 2** | Small installer (~10 MB vs 100+ MB Electron), native file & network access (plotters), calls Rust core directly |
| UI | **TypeScript + React** for panels/dialogs; **HTML Canvas 2D** (WebGL later if needed) for the drawing area | Fast to build modern UI; canvas gives full control of CAD rendering |
| Polygon offset & booleans | **Clipper2** (via Rust crate) or `i_overlay` — decided in spike S2 | Industry-proven for seam allowance & buffers |
| Nesting | **jagua-rs** (collision engine) + **sparrow** (strip-packing optimizer) — evaluated in spike S1; fallback libnest2d | Open-source, state-of-the-art results on strip packing (= fabric roll of fixed width). **Licences must be checked before commercial use.** |
| File format | JSON (`.gcad`), zipped when large | Human-readable, diffable, easy to version |
| Reports | PDF via `printpdf`/`typst`; Excel via `rust_xlsxwriter` | No Office dependency |
| Tests | `cargo test`, property tests (`proptest`), golden-file tests for DXF/HPGL, Playwright for UI smoke tests | Geometry bugs are subtle — tests are mandatory |

**Alternative considered:** C# / WPF (Windows-only). Simpler if the team already knows .NET,
but weaker nesting ecosystem and no path to web. Rust + Tauri recommended.

## 2. High-level structure

```
┌────────────────────────── Tauri desktop app ───────────────────────────┐
│  UI (TypeScript/React)                                                  │
│   ├─ Pattern editor canvas + tools                                      │
│   ├─ Grading panels (rule table, stack view)                            │
│   ├─ Marker editor canvas (bump, drag, live efficiency)                 │
│   └─ Plot queue, settings, reports                                      │
│                    │  commands (JSON)        ▲ events / render data      │
│                    ▼                         │                           │
│  Rust core (crates)                                                     │
│   ├─ geom      vectors, Bézier, flatten, offset, booleans, intersections│
│   ├─ model     Style, Piece, Point, Internals, SizeRange (document)     │
│   ├─ ops       editing commands (undo/redo), drafting & piece tools     │
│   ├─ grading   grade rules, size evaluation, rule tables                │
│   ├─ marker    marker order, constraints, manual placement, bump        │
│   ├─ nest      auto-nesting adapter (jagua-rs/sparrow), background jobs │
│   ├─ io        DXF-AAMA/ASTM + RUL, HPGL/PLT, SVG, PDF, Excel, JSON     │
│   └─ plot      plotter profiles, calibration, queue, TCP/serial drivers │
└─────────────────────────────────────────────────────────────────────────┘
```

Repository layout:

```
garment-cad/
  crates/
    geom/  model/  ops/  grading/  marker/  nest/  io/  plot/
    cli/            # command-line tool: convert, nest, plot (great for testing & batch)
  app/
    src-tauri/      # Tauri Rust side (thin: exposes core as commands)
    src/            # TypeScript UI
  testdata/         # real sample files (Richpeace DXF, PLT, markers) + golden outputs
  docs/
```

The **CLI** is built first: it lets us test import → nest → plot end to end before any UI exists,
and later runs batch/overnight nesting.

## 3. Data model (simplified)

All lengths in **millimetres (f64)**. Geometry is stored **for the base size only**; other sizes are
computed from grade rules on demand (never stored separately → no drift).

```text
Style
  id, styleNo, name, buyer, season, garmentType
  sizeRange: { sizes: ["S","M","L","XL"], base: "M" }
  measurements: [{ name: "Chest", values: { S: 980, M: 1030, ... } }]   (P2)
  ruleTables: [RuleTable]
  draft: Draft                    # construction geometry (lines, curves, points)
  pieces: [Piece]

Piece
  id, name, code, quantity, mirrorPair: bool, onFold: bool, fabricType
  outline: [Vertex]               # closed loop, sew line (net)
  seamAllowance: [{ edge: (vStart, vEnd), width, cornerType }]
  internals: [Notch | Drill | InternalLine | Text | Button | Buttonhole]
  grainLine: { a: Vec2, b: Vec2 }
  foldLine?: (vA, vB)
  markerRules: { orientation: Fixed|Rot180|Rot90|Free, flip: bool, tilt: deg, buffer: mm }

Vertex
  id, pos: Vec2, kind: Corner|Curve, handleIn?, handleOut?  # cubic Bézier handles
  grade?: { ruleRef | perStep: { "M→L": (dx,dy), ... } }

Notch
  edgeRef, position: { fromVertex, distance } | { percent }, type: Slit|V|T|U|Castle, depth, width
```

Rules:
- **Every edit is a Command** (`apply` / `undo`), so undo/redo, autosave journal, scripting and tests
  all use the same path.
- Internals reference edges by vertex ids + distance/percent, so they **follow edits and grading
  automatically**.
- Seam allowance and cut line are **derived** (recomputed), never stored.

## 4. Key algorithms

| Problem | Approach |
|---|---|
| Curves | Cubic Bézier per segment; flatten to polylines with chord tolerance 0.05 mm for offset, nesting, plotting |
| Curve length / point at distance | Adaptive subdivision (Gauss-Legendre) |
| Seam allowance | Per-edge offset of the sew line, then corner construction by type (miter / square / mirror / cut-off) → union with Clipper2 |
| Grading | For size s: vertex.pos + Σ grade steps from base to s; handles graded proportionally; then recompute derived geometry |
| Distribute grade along curve | Interpolate grade by arc-length ratio between the two graded points |
| Dart rotation | Rotate the outline segment between the old and new dart positions about the apex |
| Slash & spread | Split piece along slash lines, rotate/translate parts about hinge points, rebuild outline |
| Manual bump | Translate piece along direction until first collision: polygon-vs-polygon sweep using no-fit polygon or binary search with fast collision check (jagua-rs quadtree) |
| Auto-nesting | Strip packing: fixed width, minimize length. Pieces = flattened cut-line polygons + buffer; allowed rotations from rules. Time-limited, anytime (best-so-far) |
| Efficiency | Σ piece area ÷ (usable width × length) |
| HPGL output | Units 40 per mm (0.025 mm), `IN; SP1; PU x,y; PD x,y,...; LB text`; calibration = scale X/Y before writing |

## 5. File formats

### Native `.gcad`
JSON with `"formatVersion"`. Migration functions for every version bump (old files must always open).

### DXF-AAMA / ASTM (D6673) — import & export
Each piece is a DXF `BLOCK`. Standard layer numbers (verify against the spec + real Richpeace/Gemini files):

| Layer | Content |
|---|---|
| 1 | Piece boundary (cut line) |
| 2 | Turn (corner) points |
| 3 | Curve points |
| 4 | Notches |
| 5 | Grade reference line |
| 6 | Mirror line |
| 7 | Grain line |
| 8 | Internal lines |
| 13 | Drill holes |
| 14 | Sew line |
| 15 | Annotation text |

Grade rules come in a separate `.RUL` text file (rule number per grade point, dX/dY per size step).
Text entities in each block carry piece name, quantity, size, etc.
**Real sample files from Richpeace are the true spec** — we build tests from them.

### HPGL / PLT
Read and write. Read is needed to import external markers and to replace PlotCenter.

## 6. Plotter communication

- **Network (primary):** open TCP socket to plotter IP and raw port (commonly 9100 — confirm for
  our Richpeace model), stream HPGL in chunks, watch for errors/timeouts.
- **USB/serial (P2):** via OS serial port or Windows printer spool (raw mode).
- Calibration stored per plotter profile; applied at output time only.
- Queue persisted to disk so jobs survive app restart.

## 7. Quality & engineering practices

- Git + GitHub; every change via pull request; CI runs tests on Windows + Linux.
- Golden-file tests: `testdata/richpeace/*.dxf` → import → export → compare.
- Nesting benchmark suite: 20 real markers with known Richpeace efficiency; CI reports our efficiency.
- Release builds: signed Windows installer (MSI) via Tauri bundler; auto-update (P2).
