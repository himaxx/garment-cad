# Garment CAD (working title)

In-house garment CAD that does **pattern design + grading + marker making + plotting** in one application.
It is meant to replace Richpeace DGS, GMS and PlotCenter.

## Documents
1. [Functional specification](docs/01-functional-spec.md): every feature, with an ID and priority (M / P2 / P3)
2. [Architecture](docs/02-architecture.md): technology stack, modules, data model, file formats, algorithms
3. [Implementation plan](docs/03-implementation-plan.md): phases, milestones, go/no-go gate, timeline, risks
4. [Richpeace file formats](docs/formats/richpeace-dgs.md): reverse-engineering notes for `.dgs` / `.mkr`

## Status: Phase 0 (foundation)
- [x] Rust workspace: `geom`, `model`, `io`, `cli`
- [x] **Richpeace `.dgs` importer**: sizes, base size, pieces, quantity per size, outline, grain line and internal lines for every size
- [x] HPGL/PLT writer with X/Y calibration, and an SVG preview
- [x] `gcad` CLI: `info`, `svg`, `plt`
- [ ] DXF-AAMA import/export
- [ ] Plotter TCP driver (waiting for the plotter's IP address and port)
- [ ] Nesting spike (jagua-rs / sparrow)
- [ ] Tauri desktop app shell

## Download (Windows, no installation)
Get the latest `gcad-…-windows-x64.zip` from the [Releases page](https://github.com/himaxx/garment-cad/releases) and unzip it.
Drag a `.dgs` file onto `show-info.bat` or `make-plot.bat`. See `HOW-TO-USE.txt` inside the zip.

To publish a new version: GitHub → Actions → "Release (Windows)" → Run workflow → enter a version such as `v0.1.1`.

## Build from source & use
Requires Rust (stable).

```bash
cargo build --release
```

```bash
./target/release/gcad info  path/to/GT_102.dgs
```

```bash
./target/release/gcad svg   path/to/GT_102.dgs -o gt102.svg --size all
```

```bash
./target/release/gcad plt   path/to/GT_102.dgs -o gt102.plt --size 28 --paper-width 1600
```

Plotter calibration: plot a file, measure a long line, then pass `--scale-x <expected/measured>` (and `--scale-y`).

## Tests
```bash
cargo test
```

Tests that use real factory files run when sample files are placed in `testdata/private/` (git-ignored, because this repo is public):

```bash
GCAD_SAMPLES=$PWD/testdata/private cargo test
```
