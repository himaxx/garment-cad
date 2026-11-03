//! Reader for Richpeace DGS pattern files (`.dgs`, magic `EGPR`).
//!
//! The format is proprietary and undocumented; this reader is based on reverse-engineering
//! sample files (see docs/formats/richpeace-dgs.md). It extracts the **evaluated geometry**
//! Richpeace stores for every size (outline, grain line, internal lines), the size table and
//! piece quantities. It does not (yet) recover construction points, curve handles, grade rules,
//! notches or seam-allowance settings.
//!
//! File layout (byte-packed little-endian, no alignment):
//! - header with magic `EGPR`
//! - size table: `u32 count`, then per size `u8 len, name[len], rgb[3], tail[10]`
//!   (a non-zero tail marks the base size)
//! - section 1: per piece a length-prefixed name, quantities per size and the construction
//!   points (84-byte point records: 8 sizes × (x, y))
//! - section 2: the same piece names again, followed by polyline records
//!   `[a, b, c, d: i32][n: i32][n × (x: i32, y: i32)]` where the 4-word prefix tells the kind:
//!   `(1,1,0,_)` = outline of one size, `(_,_,1,4)` with n = 2 = grain line,
//!   `(0,1,_,_)` = internal line.
//!
//! Coordinates are integers in 0.01 mm.

use gcad_geom::Vec2;
use gcad_model::{GrainLine, Piece, PieceGeometry, Size, Style};

use crate::ImportError;

const MAGIC: &[u8; 4] = b"EGPR";
/// File units are 1/100 mm.
pub const UNIT_MM: f64 = 0.01;
const MAX_COORD: i32 = 3_000_000; // 30 m — anything larger is not a coordinate
const MAX_SIZES: u32 = 40;

fn u32_at(d: &[u8], i: usize) -> Option<u32> {
    d.get(i..i + 4)
        .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
}
fn i32_at(d: &[u8], i: usize) -> Option<i32> {
    u32_at(d, i).map(|v| v as i32)
}

fn is_name_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b" _-./()#&+'".contains(&b)
}

/// Parse a size table at offset `i`. Returns sizes and the end offset.
fn parse_size_table(d: &[u8], i: usize) -> Option<(Vec<Size>, usize)> {
    let count = u32_at(d, i)?;
    if count == 0 || count > MAX_SIZES {
        return None;
    }
    let mut p = i + 4;
    let mut sizes = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let len = *d.get(p)? as usize;
        if len == 0 || len > 12 {
            return None;
        }
        let name = d.get(p + 1..p + 1 + len)?;
        if !name.iter().all(|b| b.is_ascii_graphic()) {
            return None;
        }
        let tail = d.get(p + 1 + len + 3..p + 1 + len + 13)?;
        sizes.push(Size {
            name: String::from_utf8_lossy(name).into_owned(),
            base: tail.iter().any(|b| *b != 0),
        });
        p += 1 + len + 13;
    }
    // names must be unique
    for (k, s) in sizes.iter().enumerate() {
        if sizes[..k].iter().any(|o| o.name == s.name) {
            return None;
        }
    }
    Some((sizes, p))
}

fn find_size_table(d: &[u8]) -> Option<(Vec<Size>, usize)> {
    let limit = d.len().min(16 * 1024);
    let mut best: Option<(Vec<Size>, usize)> = None;
    for i in 4..limit {
        if let Some((sizes, end)) = parse_size_table(d, i) {
            if best.as_ref().map_or(true, |(b, _)| sizes.len() > b.len()) {
                best = Some((sizes, end));
            }
        }
    }
    best
}

/// Length-prefixed names (`0x00, len, chars…, 0x00`) after `from`.
fn find_names(d: &[u8], from: usize) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut i = from.max(1);
    while i + 2 < d.len() {
        let len = d[i] as usize;
        if d[i - 1] == 0 && (2..=40).contains(&len) && i + 1 + len < d.len() {
            let s = &d[i + 1..i + 1 + len];
            if s[0].is_ascii_alphanumeric()
                && s.iter().all(|b| is_name_byte(*b))
                && d[i + 1 + len] == 0
            {
                out.push((i, String::from_utf8_lossy(s).trim_end().to_string()));
                i += 1 + len;
                continue;
            }
        }
        i += 1;
    }
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Outline,
    Grain,
    Internal,
    Other,
}

struct PolyRecord {
    kind: Kind,
    points: Vec<Vec2>,
}

fn classify(prefix: [i32; 4], n: usize) -> Kind {
    match prefix {
        [1, 1, 0, _] => Kind::Outline,
        [_, _, 1, 4] if n == 2 => Kind::Grain,
        [0, 1, _, _] => Kind::Internal,
        _ => Kind::Other,
    }
}

/// Scan `[start, end)` for polyline records.
fn scan_polylines(d: &[u8], start: usize, end: usize) -> Vec<PolyRecord> {
    let mut out = Vec::new();
    let mut i = start.max(16);
    while i + 4 <= end {
        let n = i32_at(d, i).unwrap_or(0);
        if (2..=20_000).contains(&n) {
            let n = n as usize;
            let body = i + 4;
            if let Some(raw) = d.get(body..body + n * 8) {
                let coords: Vec<i32> = raw
                    .chunks_exact(4)
                    .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                    .collect();
                let plausible = coords.iter().all(|v| v.abs() < MAX_COORD);
                let distinct = {
                    let mut v = coords.clone();
                    v.sort_unstable();
                    v.dedup();
                    v.len() > 2
                };
                // Both coordinates of the first point must look like real positions (> 10 mm from
                // the origin); otherwise small header integers (e.g. the `4` before a grain line)
                // are mistaken for a record start and swallow the real record.
                let anchored = coords[0].abs() > 1000 && coords[1].abs() > 1000;
                if plausible && distinct && anchored {
                    let prefix = [
                        i32_at(d, i - 16).unwrap_or(-9),
                        i32_at(d, i - 12).unwrap_or(-9),
                        i32_at(d, i - 8).unwrap_or(-9),
                        i32_at(d, i - 4).unwrap_or(-9),
                    ];
                    let points = coords
                        .chunks_exact(2)
                        .map(|c| Vec2::new(c[0] as f64 * UNIT_MM, c[1] as f64 * UNIT_MM))
                        .collect();
                    out.push(PolyRecord {
                        kind: classify(prefix, n),
                        points,
                    });
                    i = body + n * 8;
                    continue;
                }
            }
        }
        i += 1;
    }
    out
}

/// Quantity per size: first `u32 == nsizes` followed by `nsizes` small values.
fn find_quantities(d: &[u8], start: usize, end: usize, nsizes: usize) -> Option<Vec<u32>> {
    let mut i = start;
    while i + 4 * (nsizes + 1) <= end {
        if u32_at(d, i) == Some(nsizes as u32) {
            let q: Vec<u32> = (0..nsizes)
                .filter_map(|k| u32_at(d, i + 4 + 4 * k))
                .collect();
            if q.len() == nsizes && q.iter().all(|v| (1..=100).contains(v)) {
                return Some(q);
            }
        }
        i += 1;
    }
    None
}

/// Read a Richpeace `.dgs` file from memory.
pub fn read(d: &[u8], style_name: &str) -> Result<Style, ImportError> {
    if d.len() < 64 || &d[0..4] != MAGIC {
        return Err(ImportError::NotRecognised(
            "missing EGPR header — not a Richpeace DGS file".into(),
        ));
    }
    let (sizes, table_end) = find_size_table(d)
        .ok_or_else(|| ImportError::Unsupported("size table not found".into()))?;
    let ns = sizes.len();

    // Piece names are stored twice (section 1 and section 2). Find the repeated tail.
    let names = find_names(d, table_end);
    let k = (1..=names.len() / 2)
        .rev()
        .find(|&k| {
            let n = names.len();
            names[n - k..]
                .iter()
                .zip(&names[n - 2 * k..n - k])
                .all(|(a, b)| a.1 == b.1)
        })
        .ok_or_else(|| ImportError::Unsupported("piece name sections not found".into()))?;
    let sec1 = &names[names.len() - 2 * k..names.len() - k];
    let sec2 = &names[names.len() - k..];

    let mut style = Style {
        name: style_name.to_string(),
        sizes,
        pieces: Vec::new(),
        warnings: Vec::new(),
    };

    for j in 0..k {
        let name = sec2[j].1.clone();

        let q_start = sec1[j].0;
        let q_end = if j + 1 < k { sec1[j + 1].0 } else { sec2[0].0 };
        let quantity = find_quantities(d, q_start, q_end, ns).unwrap_or_else(|| {
            style
                .warnings
                .push(format!("{name}: quantity not found, assuming 1"));
            vec![1; ns]
        });

        let g_start = sec2[j].0;
        let g_end = if j + 1 < k { sec2[j + 1].0 } else { d.len() };
        let recs = scan_polylines(d, g_start, g_end);

        let outlines: Vec<&PolyRecord> = recs.iter().filter(|r| r.kind == Kind::Outline).collect();
        let grains: Vec<&PolyRecord> = recs.iter().filter(|r| r.kind == Kind::Grain).collect();
        let internals: Vec<&PolyRecord> =
            recs.iter().filter(|r| r.kind == Kind::Internal).collect();

        if outlines.len() < ns {
            style.warnings.push(format!(
                "{name}: found {} outlines for {ns} sizes — piece skipped",
                outlines.len()
            ));
            continue;
        }
        if outlines.len() > ns {
            style.warnings.push(format!(
                "{name}: {} extra outline record(s) after the piece were ignored (unknown object)",
                outlines.len() - ns
            ));
        }
        if grains.len() < ns {
            style
                .warnings
                .push(format!("{name}: grain line missing for some sizes"));
        }
        let lines_per_size = internals.len() / ns;
        if internals.len() % ns != 0 {
            style.warnings.push(format!(
                "{name}: {} internal lines do not divide into {ns} sizes — internals ignored",
                internals.len()
            ));
        }

        let geo = (0..ns)
            .map(|s| PieceGeometry {
                outline: outlines[s].points.clone(),
                grain: grains.get(s).map(|g| GrainLine {
                    a: g.points[0],
                    b: g.points[1],
                }),
                internals: if internals.len() % ns == 0 {
                    (0..lines_per_size)
                        .map(|l| internals[l * ns + s].points.clone())
                        .collect()
                } else {
                    Vec::new()
                },
            })
            .collect();

        style.pieces.push(Piece {
            name,
            quantity,
            sizes: geo,
        });
    }

    if style.pieces.is_empty() {
        return Err(ImportError::Unsupported("no pieces could be read".into()));
    }
    Ok(style)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_dgs() {
        assert!(matches!(
            read(
                b"hello world, definitely not a pattern file at all......................",
                "x"
            ),
            Err(ImportError::NotRecognised(_))
        ));
    }

    #[test]
    fn parses_size_table() {
        let mut d = vec![0u8; 4];
        d.extend_from_slice(&3u32.to_le_bytes());
        for (name, base) in [("S", false), ("M", true), ("XL", false)] {
            d.push(name.len() as u8);
            d.extend_from_slice(name.as_bytes());
            d.extend_from_slice(&[0, 0, 0]);
            let mut tail = [0u8; 10];
            if base {
                tail[1] = 1;
            }
            d.extend_from_slice(&tail);
        }
        let (sizes, end) = parse_size_table(&d, 4).unwrap();
        assert_eq!(end, d.len());
        assert_eq!(
            sizes.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
            ["S", "M", "XL"]
        );
        assert_eq!(sizes.iter().position(|s| s.base), Some(1));
    }

    #[test]
    fn classifies_records() {
        assert_eq!(classify([1, 1, 0, 1], 66), Kind::Outline);
        assert_eq!(classify([88930, 0, 1, 4], 2), Kind::Grain);
        assert_eq!(classify([0, 1, 0, 0], 15), Kind::Internal);
        assert_eq!(classify([7, 7, 7, 7], 3), Kind::Other);
    }

    /// Runs only when GCAD_SAMPLES points at a folder containing GT_102.dgs.
    #[test]
    fn reads_sample_gt102() {
        let Ok(dir) = std::env::var("GCAD_SAMPLES") else {
            return;
        };
        let path = std::path::Path::new(&dir).join("GT_102.dgs");
        let Ok(bytes) = std::fs::read(&path) else {
            return;
        };
        let style = read(&bytes, "GT_102").unwrap();
        let sizes: Vec<_> = style.sizes.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(sizes, ["22", "24", "26", "28", "30", "32", "34", "36"]);
        assert_eq!(style.sizes[style.base_size_index()].name, "28");
        let names: Vec<_> = style.pieces.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["NECK  PATTI", "SLEEVE  NEW", "BACK", "FERNT"]);
        let neck = &style.pieces[0];
        assert_eq!(neck.quantity, vec![1; 8]);
        assert_eq!(style.pieces[1].quantity, vec![2; 8]);
        // Neck patti is a 57.15 mm (2.25") wide strip that grows 16.26 mm per size.
        let b0 = neck.sizes[0].bbox();
        let b1 = neck.sizes[1].bbox();
        assert!((b0.width() - 57.15).abs() < 0.01);
        assert!((b1.height() - b0.height() - 16.26).abs() < 0.01);
        assert!(style
            .pieces
            .iter()
            .all(|p| p.sizes.iter().all(|g| g.grain.is_some())));
        assert_eq!(style.pieces[3].sizes[0].internals.len(), 2);
    }
}
