# Richpeace native file formats — reverse-engineering notes

Status: **partial, based on 1 DGS + 1 MKR sample** (GT_102.dgs, Untitled.mkr; Richpeace V9/V10 era,
files from 2026). The samples are stored locally in `testdata/private/` and are not committed.
Everything below must be re-checked against more samples.

All integers are little-endian. The files are **byte-packed** (records start at any byte offset).

## `.dgs` — pattern file (DGS)

| Item | Finding |
|---|---|
| Magic | `EGPR` at offset 0 |
| Units | integer **0.01 mm** (neck patti width 5715 = 57.15 mm = 2¼″) |
| Axes | X/Y as drawn in DGS; garments are drawn with body length along X and grain along X |
| Sizes | `u32 count`, then per size: `u8 len`, `name[len]`, `rgb[3]` (size colour), `tail[10]`; a non-zero tail marks the **base size** (sample: 22 24 26 **28** 30 32 34 36) |
| Font | length-prefixed font name (`SimSun`) before the pieces |
| Piece names | length-prefixed ASCII, stored **twice**: once in section 1 and once in section 2 |

### Section 1: construction data (per piece)
- Name, then `u32 nsizes` + `nsizes × u32` **quantity per size** (Sleeve = 2, others = 1).
- **Point records, 84 bytes each:** `i32 1, u32 attr/colour, u32 type, 8 × (i32 x, i32 y), i32 0, i32 0`.
  Each point stores its position **in every size**, so grading is stored as evaluated positions
  rather than as rules.
  `type` 1 shows up on corner/grade points and 0 on curve points (needs confirming).
- After the points comes a topology block (lines referencing point indices, curve flags…). **Not decoded yet.**

### Section 2: evaluated geometry (per piece). **Used by the importer.**
Polyline record: `[a, b, c, d: i32][n: i32][n × (x: i32, y: i32)]`

| Prefix `(a,b,c,d)` | Meaning | Count |
|---|---|---|
| `(1,1,0,_)` | Piece **outline**, one per size (flattened curve, 4–184 points). When `d = 1`, an identical copy follows immediately and is skipped. | nsizes |
| `(_,_,1,4)` with `n = 2` | **Grain line** per size | nsizes |
| `(0,1,_,_)` | **Internal line**: line *L* of size *S* is record `L × nsizes + S` | k × nsizes |

Observed oddities:
- After the last piece there is an extra unnamed object (8 identical 96-point outlines at negative
  coordinates, about 370 × 29 mm). It is ignored for now and the importer warns about it.
- In GT_102, Front sizes 22/24/26 are stored **rotated about 30°** compared with sizes 28–36, while their grain
  line is not rotated. Still to check in DGS whether those sizes really look rotated.

### Not found / not decoded yet
- Notches, drill holes, buttonholes (they may be inside the internal lines or the section-1 topology)
- Seam allowance widths, and whether the outline is the cut line or the sew line
- Mirror/fold flags, fabric type, piece annotations
- Grade rules (probably not stored; DGS keeps the evaluated per-size points)

## `.mkr` — marker file (GMS)

| Item | Finding |
|---|---|
| Magic | `RPMF`; an embedded `EGPR` block follows at ~0x17C |
| Source link | Full path of the source `.dgs` plus load/modified time, file length and FileID as text (`\r\n` separated) |
| Sizes | `u32 count`, then per size `u32 len, name, …` (base flag on 28) |
| Pieces | One record per piece per size (name repeated), followed by geometry stored **relative to the piece** in a different layout (blocks of 256 values seen) |
| Header numbers | `57432` (u32) and `44.2076` (f64) near 0xF4; two `10000.0` doubles. Probably marker length/width/efficiency, but **not confirmed** |

The sample marker was made from **GT 203**, a different style from the GT_102 pattern sample.

**Next step:** ask for one marker together with its exact `.dgs`, plus the length, width and efficiency that GMS shows
for it, so the header fields can be identified.

## Recommendation
Keep the native `.dgs` importer for bulk migration of the existing library, because it already recovers what plotting
and nesting need. Use **DXF-AAMA export from DGS** as the reference for notches, seam allowance and piece
attributes, and compare the two importers on the same styles.
