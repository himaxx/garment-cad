# Functional Specification — Garment CAD (working title)

One desktop application that covers the full garment CAD workflow:

```
Pattern Design ──► Grading ──► Marker / Nesting ──► Plot & Cut
     (DGS)          (DGS)            (GMS)           (PlotCenter)
```

Reference product: Richpeace DGS + GMS + PlotCenter. Goal: replace it for in-house
use first, with room to grow into a commercial product.

**Priority legend**
- **M** = MVP (needed to replace Richpeace in our own factory)
- **P2** = second release (important, not blocking go-live)
- **P3** = later / nice to have

Each feature has an ID (e.g. `PD-08`) so plan tasks, tests and bugs can refer to it.

---

## 0. Users and their jobs

| User | Main job in the software |
|---|---|
| Pattern master | Drafts patterns, edits pieces, adds seam allowance, notches, grading |
| Marker maker / CAD operator | Creates marker orders, runs nesting, checks efficiency, sends to plotter |
| Cutting supervisor | Gets plotted markers, cut plans, piece/bundle counts |
| Merchandiser / costing | Gets fabric consumption per garment and per order |
| Admin | Plotter setup, settings, users, backups |

---

## 1. Core concepts (glossary)

| Term | Meaning |
|---|---|
| **Style** | One garment design (e.g. "Men's formal shirt, style 1042"). Contains pieces, size range, measurements, grade rules. |
| **Piece** | One pattern piece (Front, Back, Sleeve, Collar…). Has a closed outline, internals, grain line, seam allowance, quantity. |
| **Point** | A vertex on a line. Types: *corner* (turn point), *curve point*, *grade point* (carries a grade rule). |
| **Sew line / Cut line** | Sew line = net pattern edge; cut line = sew line + seam allowance. |
| **Internals** | Notches, drill holes, buttonholes, internal lines, text, grain line, fold/mirror line. |
| **Size range** | Ordered sizes, e.g. S–M–L–XL–XXL, with one **base size**. |
| **Grade rule** | dX/dY movement of a grade point for each size step relative to base. |
| **Rule table** | A named library of grade rules reused across styles. |
| **Marker** | Arrangement of all pieces for an order ratio on a fabric width. Output: length + efficiency %. |
| **Marker order** | Style(s) + size ratio (e.g. S1 M2 L2 XL1) + fabric spec + constraints. |
| **Efficiency** | Area of pieces ÷ (fabric width × marker length) × 100. |
| **Lay / Spread** | Fabric plies stacked on the cutting table; one marker is cut through all plies. |
| **HPGL / PLT** | Plotter language/file used by inkjet and cutting plotters. |
| **DXF-AAMA / ASTM** | Industry exchange format for pattern pieces (+ `.RUL` file for grade rules). |

---

## 2. Module A — Style & Library Management

| ID | Feature | Pri |
|---|---|---|
| ST-01 | Create / open / save / save-as style files; recent files list | M |
| ST-02 | Style header: style no., name, buyer, season, garment type, remarks | M |
| ST-03 | Piece list panel: name, code, quantity per garment, mirror (L/R pair), fold, fabric type (shell/lining/fusing/contrast), on/off for marker | M |
| ST-04 | Size range editor: add/remove/reorder sizes, choose base size | M |
| ST-05 | Autosave + crash recovery | M |
| ST-06 | Style library browser with search (by style no., buyer, type) and thumbnails | P2 |
| ST-07 | Version history of a style (who/when/what changed), restore old version | P2 |
| ST-08 | Templates: start a new style from a base block (shirt, trouser, kurta, T-shirt) | P2 |
| ST-09 | Copy pieces between styles | M |
| ST-10 | Multi-user central library on a shared server/NAS, file locking | P3 |

---

## 3. Module B — Pattern Design (DGS equivalent)

### B1. Workspace & navigation
| ID | Feature | Pri |
|---|---|---|
| PD-01 | Infinite canvas: zoom (wheel, window, fit all, fit selection), pan, 1:1 real-size zoom | M |
| PD-02 | Units: cm / mm / inch (with fractions 1/8, 1/16); internal unit always mm | M |
| PD-03 | Snapping: endpoint, midpoint, intersection, on-curve, perpendicular, grid; toggle with key | M |
| PD-04 | **Numeric input everywhere**: type length, angle, offset (pattern masters work by numbers, not mouse) | M |
| PD-05 | Unlimited undo/redo | M |
| PD-06 | Selection: click, window, crossing, select-all-of-type; properties panel for selected object | M |
| PD-07 | Layers / visibility: construction lines, seam allowance, internals, grading, text | M |
| PD-08 | Rulers, grid, crosshair, coordinate display | M |
| PD-09 | Keyboard shortcuts, customizable; tool palette | M |
| PD-10 | "Smart pen": one multi-purpose tool (draw line/curve, connect, trim, delete by context) like Richpeace intelligent pen | P2 |

### B2. Drafting tools (construction geometry)
| ID | Feature | Pri |
|---|---|---|
| PD-20 | Point: by click, by coordinates, by offset from point (dx, dy), by distance along line/curve, at intersection | M |
| PD-21 | Straight line / polyline (length + angle input) | M |
| PD-22 | Curve (cubic Bézier/spline) with handles; edit curve shape; smooth curve | M |
| PD-23 | Rectangle (body block), circle, arc | M |
| PD-24 | Parallel/offset line, perpendicular, angle bisector, tangent | M |
| PD-25 | Trim, extend, split at point, join lines | M |
| PD-26 | Move, copy, rotate (about point / by angle), mirror (about line), scale | M |
| PD-27 | Fix curve length: adjust curve to a target length keeping shape | P2 |
| PD-28 | Divide line/curve into N equal parts | M |
| PD-29 | Formula/parametric drafting: points defined by formulas on measurement variables (e.g. `chest/4 + 1.5`); changing measurements regenerates the draft | P3 |

### B3. Measuring & checking
| ID | Feature | Pri |
|---|---|---|
| PD-40 | Tape measure: point-to-point distance, length along line/curve | M |
| PD-41 | Compare lengths: e.g. armhole vs sleeve cap, front vs back side seam; shows difference (ease) | M |
| PD-42 | Walk pattern: walk one piece's seam along another to check match & notch positions | P2 |
| PD-43 | Angle measure, piece area & perimeter | M |
| PD-44 | Measurement table per style that updates live after edits (POM list) | P2 |

### B4. Piece creation & management
| ID | Feature | Pri |
|---|---|---|
| PD-60 | Create piece by picking lines that form a closed outline from the draft | M |
| PD-61 | Create piece from imported outline (DXF/digitizer) | M |
| PD-62 | Piece properties: name, quantity, mirror pair, fold, fabric type, piece annotation | M |
| PD-63 | Fold line: mirror half piece to full (unfold) and back | M |
| PD-64 | Split piece along a line (e.g. yoke from back) — with or without seam allowance added | M |
| PD-65 | Combine/join two pieces along matching edges | P2 |
| PD-66 | Copy / duplicate piece | M |
| PD-67 | Auto-arrange pieces on screen | M |

### B5. Piece modification
| ID | Feature | Pri |
|---|---|---|
| PD-80 | Move point(s) with numeric dx/dy; neighbors optionally follow smoothly | M |
| PD-81 | Add / delete point, change point type (corner/curve) | M |
| PD-82 | Edit curve with handles; keep tangent continuity at curve points | M |
| PD-83 | Lengthen / shorten piece between two lines (with balanced side seams) | M |
| PD-84 | Rotate / flip piece, align to grain | M |
| PD-85 | Shrinkage compensation per piece/fabric (warp % / weft %) | M |

### B6. Darts
| ID | Feature | Pri |
|---|---|---|
| PD-100 | Add dart (position on edge, width, length/apex) | M |
| PD-101 | Rotate / transfer dart to another location (pivot about apex) | P2 |
| PD-102 | Close dart / open dart | P2 |
| PD-103 | Split dart into multiple darts | P2 |
| PD-104 | Dart cap (true the edge so the folded dart sews straight) | P2 |
| PD-105 | Convert dart into gathers/fullness | P3 |

### B7. Pleats, tucks & fullness
| ID | Feature | Pri |
|---|---|---|
| PD-120 | Knife pleat, box pleat, inverted pleat (depth, direction, count) | P2 |
| PD-121 | Tapered pleat; pleat cap (true edge) | P2 |
| PD-122 | Slash & spread — parallel (add length) and fan (add flare) | P2 |
| PD-123 | Slash & close (remove fullness) | P3 |
| PD-124 | Gathers marking (gathering ratio) | P3 |

### B8. Seam allowance
| ID | Feature | Pri |
|---|---|---|
| PD-140 | Seam allowance per edge (different widths per edge), default per style | M |
| PD-141 | Corner types: mitered/extended, square (right-angle), mirror/reverse (for hems that fold back), cut-off, step | M |
| PD-142 | Hem with folded turnback (allowance mirrors piece shape) | P2 |
| PD-143 | Auto-update allowance after any edit and for every graded size | M |
| PD-144 | Show/hide sew line vs cut line; cut-line-only pieces (allowance included) | M |

### B9. Notches
| ID | Feature | Pri |
|---|---|---|
| PD-160 | Notch on edge: by click, by distance from a point, by % of edge length | M |
| PD-161 | Notch types: slit (I), V, T, U, box/castle; depth & width | M |
| PD-162 | Notch on sew line projected to cut line, perpendicular to edge | M |
| PD-163 | Notches move with edits and with grading (keep distance or % rule) | M |
| PD-164 | Matching notch: place a notch on piece B at the same seam distance as on piece A | P2 |

### B10. Internals & annotation
| ID | Feature | Pri |
|---|---|---|
| PD-180 | Grain line (with direction arrows), bias lines | M |
| PD-181 | Drill holes (inset from edge), button & buttonhole placement with equal spacing | M |
| PD-182 | Internal lines (pocket position, placement lines, stitch lines) | M |
| PD-183 | Text labels; automatic piece label (style, piece name, size, qty, fabric) | M |
| PD-184 | Mirror / fold line display | M |

### B11. Derived pieces
| ID | Feature | Pri |
|---|---|---|
| PD-200 | Facing from piece edge (width along edge) | P2 |
| PD-201 | Lining from shell (with ease/pleat additions) | P2 |
| PD-202 | Fusing/interlining (inset from cut line) | P2 |
| PD-203 | Derived pieces stay linked to parent and update when parent changes | P3 |

### B12. Pattern input
| ID | Feature | Pri |
|---|---|---|
| PD-220 | Import DXF-AAMA / DXF-ASTM (+ `.RUL` grade rules) — **including files exported by Richpeace** | M |
| PD-221 | Import HPGL/PLT outlines as pieces | P2 |
| PD-222 | Digitizer input (serial/USB digitizer: corner, curve, notch, grain, grade points) | P2 |
| PD-223 | Photo digitizing: photo of paper pattern on a calibrated board → auto outline | P3 |
| PD-224 | Import plain DXF (AutoCAD) lines/polylines | P2 |

---

## 4. Module C — Grading

| ID | Feature | Pri |
|---|---|---|
| GR-01 | Size chart setup: sizes, base size, break sizes (different step between sizes) | M |
| GR-02 | Point grading: enter dX/dY per size step for a grade point (incremental or cumulative entry) | M |
| GR-03 | Rule table library: save named rules, assign rule to point by number | M |
| GR-04 | Copy / paste grade; paste with flip X / flip Y / swap X-Y | M |
| GR-05 | Distribute grade along a curve between two grade points (proportional) | M |
| GR-06 | Nest/stack view: all sizes overlaid, aligned at a chosen point or on grain | M |
| GR-07 | Show one size / selected sizes; extract any size as a separate piece | M |
| GR-08 | Graded measurement report: length of selected lines/curves for every size (Excel export) | M |
| GR-09 | Notches, drills, internals and seam allowance follow grading | M |
| GR-10 | Grade construction lines (so a re-generated piece grades too) | P2 |
| GR-11 | Measurement-driven auto grading from a size chart (POM increments) | P3 |
| GR-12 | Import size chart from Excel | P2 |
| GR-13 | Grade rule export/import with DXF-AAMA `.RUL` | M |

---

## 5. Module D — Marker / Nesting (GMS equivalent)

### D1. Marker order setup
| ID | Feature | Pri |
|---|---|---|
| MK-01 | Choose style(s) and fabric type (shell / lining / fusing → separate markers) | M |
| MK-02 | Size ratio / bundle quantities (e.g. S1 M2 L2 XL1) | M |
| MK-03 | Fabric width, usable width (selvedge margins), max marker length | M |
| MK-04 | Shrinkage % warp/weft applied at marker time | M |
| MK-05 | Plies (for consumption calc), end allowance per lay | M |
| MK-06 | Multi-style marker (combine two styles on one marker) | P2 |
| MK-07 | Save marker order templates per buyer/fabric | P2 |

### D2. Piece constraints
| ID | Feature | Pri |
|---|---|---|
| MK-20 | Orientation rule per piece: grain fixed (0° only), 0°/180°, any 90°, free | M |
| MK-21 | Flip (mirror) allowed / not allowed | M |
| MK-22 | Tilt tolerance (± small angle off grain) | P2 |
| MK-23 | Buffer / gap: global and per piece | M |
| MK-24 | Nap / one-way fabric: all pieces of a garment (or whole marker) same direction | M |
| MK-25 | Folded fabric marker / tubular (knit) marker; pieces on fold placed at edge | P2 |
| MK-26 | Stripe & check (plaid) matching: repeat size, matching points, pieces snap to repeat | P3 |
| MK-27 | Fabric defect zones / splice zones | P3 |

### D3. Manual & interactive nesting
| ID | Feature | Pri |
|---|---|---|
| MK-40 | Drag pieces from unplaced bin onto marker | M |
| MK-41 | **Bump**: push piece left/right/up/down until it touches (arrow keys) — core marker-maker tool | M |
| MK-42 | Rotate/flip placed piece within allowed rules | M |
| MK-43 | Real-time overlap & out-of-bounds highlight | M |
| MK-44 | Live length, efficiency %, placed/total pieces counter | M |
| MK-45 | Lock placed pieces, then auto-nest the rest (interactive) | M |
| MK-46 | Split a piece for marker (with seam allowance added at split) | P2 |
| MK-47 | Two markers side by side for comparison | P2 |

### D4. Automatic nesting
| ID | Feature | Pri |
|---|---|---|
| MK-60 | Auto-nest with time limit (e.g. 1 / 5 / 15 min); shows best-so-far and lets user stop | M |
| MK-61 | Runs in background; UI stays responsive | M |
| MK-62 | Batch nesting: queue many marker orders (e.g. overnight) | P2 |
| MK-63 | Target: efficiency equal to or better than Richpeace GMS on our benchmark markers | M |

### D5. Marker results & output
| ID | Feature | Pri |
|---|---|---|
| MK-80 | Marker info: length, width, efficiency, piece count, consumption per garment | M |
| MK-81 | Marker report (PDF): miniature marker picture + all numbers | M |
| MK-82 | Send marker to plotter / export PLT, DXF, PDF | M |
| MK-83 | Cut order planning: split an order (size × quantity) into optimal markers & lays given table length & max plies | P3 |

---

## 6. Module E — Plot & Cut Output (PlotCenter equivalent)

| ID | Feature | Pri |
|---|---|---|
| PL-01 | Export HPGL/PLT (pen up/down, lines, text, dashed lines) | M |
| PL-02 | Send directly to network plotter (TCP/IP raw port) — **Richpeace plotters** first | M |
| PL-03 | Send to USB/serial/Windows printer port | P2 |
| PL-04 | Plotter profiles: name, IP/port, paper width, origin, units, mirror | M |
| PL-05 | Calibration: X and Y scale correction factor (plot a 1 m test square, measure, enter) | M |
| PL-06 | Pen/line mapping: cut line, sew line, internals, text → pen or line style | M |
| PL-07 | Plot queue: add, reorder, pause, cancel, re-plot; history | M |
| PL-08 | Pattern plot: plot pieces 1:1 with labels, all sizes nested or one size per piece | M |
| PL-09 | Long marker split into sections with overlap & joining marks | P2 |
| PL-10 | Cutting plotter: separate draw and cut layers, inner contours cut first, notch cut style, overcut | P2 |
| PL-11 | PDF export: full-size (for print shops) and tiled A4/A3 with alignment marks | P2 |
| PL-12 | Barcode/QR on pieces and markers (link to order) | P3 |
| PL-13 | Import external PLT files and plot them (replace PlotCenter fully) | M |

---

## 7. Module F — Reports, Exchange & Integration

| ID | Feature | Pri |
|---|---|---|
| RP-01 | Piece list report (piece, qty, fabric, area, perimeter) | M |
| RP-02 | Graded measurement report (Excel/CSV) | M |
| RP-03 | Marker report & consumption per order (PDF/Excel) | M |
| RP-04 | Export DXF-AAMA/ASTM + RUL (send patterns to buyers / other CAD) | M |
| RP-05 | Export SVG/PDF of pieces | P2 |
| RP-06 | Costing sheet: fabric consumption × rate per style | P2 |
| RP-07 | ERP integration: read orders (size breakdown) and write marker consumption back (REST API or file drop) | P3 |

---

## 8. Module G — System

| ID | Feature | Pri |
|---|---|---|
| SY-01 | Settings: units, default seam allowance, notch type, buffer, plotter defaults | M |
| SY-02 | Language: English first; Hindi / regional later | P2 |
| SY-03 | Users & roles (pattern master, marker maker, viewer) | P3 |
| SY-04 | Licensing / activation (only if sold commercially) | P3 |
| SY-05 | Error logging & crash report | M |

---

## 9. Non-functional requirements

| Area | Requirement |
|---|---|
| Precision | Internal geometry in mm, 64-bit float; display rounding only. Geometric tolerance 0.01 mm. |
| Plot accuracy | ≤ ±1 mm per metre after calibration |
| Performance | Canvas stays smooth (60 fps) with 50 pieces × 8 sizes; marker with 300 pieces interactive bump < 50 ms |
| Nesting quality | ≥ Richpeace GMS efficiency on our 20-marker benchmark set (go/no-go gate) |
| Platform | Windows 10/11 64-bit (factory PCs); works fully offline |
| Reliability | Autosave every 2 min; no data loss on crash; file format versioned and backward compatible |
| Compatibility | Round-trip DXF-AAMA with Richpeace, Gemini, Gerber files without losing pieces/notches/grades |
| Usability | Every tool usable with numeric keyboard input; shortcuts close to Richpeace habits |

---

## 10. Out of scope (for now)

- 3D garment simulation / virtual fitting (use CLO 3D alongside if needed)
- Proprietary auto-cutter formats (Gerber/Lectra cutter files) — only HPGL/DXF
- Embroidery / print design
- Full PLM
