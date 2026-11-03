//! A plot sheet: device-independent drawing (mm) that is written to HPGL, SVG, PDF…
//!
//! Coordinates: X runs along the paper roll (length), Y across the paper (width).

use gcad_geom::{BBox, Transform, Vec2};
use gcad_model::{GrainLine, PieceGeometry, Style};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pen {
    /// Piece outline (cut line).
    Cut = 1,
    /// Internal lines, grain line.
    Internal = 2,
    /// Labels.
    Text = 3,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Polyline {
        pen: Pen,
        closed: bool,
        points: Vec<Vec2>,
    },
    /// Text anchored at its lower-left corner; `height` in mm.
    Text {
        pos: Vec2,
        height: f64,
        text: String,
    },
}

#[derive(Debug, Clone, Default)]
pub struct Sheet {
    /// Usable paper width (Y extent), mm.
    pub width: f64,
    pub shapes: Vec<Shape>,
}

impl Sheet {
    pub fn bbox(&self) -> BBox {
        let mut b = BBox::empty();
        for s in &self.shapes {
            match s {
                Shape::Polyline { points, .. } => b.union(&BBox::of(points)),
                Shape::Text { pos, height, text } => {
                    b.include(*pos);
                    b.include(*pos + Vec2::new(text.len() as f64 * height * 0.7, *height));
                }
            }
        }
        b
    }
    /// Plotted length along the roll, mm.
    pub fn length(&self) -> f64 {
        let b = self.bbox();
        if b.is_empty() {
            0.0
        } else {
            b.max.x
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatternPlotOptions {
    /// Usable paper width, mm.
    pub paper_width: f64,
    /// Gap between pieces, mm.
    pub gap: f64,
    /// Label text height, mm.
    pub text_height: f64,
}

impl Default for PatternPlotOptions {
    fn default() -> Self {
        Self {
            paper_width: 1600.0,
            gap: 20.0,
            text_height: 8.0,
        }
    }
}

/// Lay out one copy of every piece in the chosen sizes for a 1:1 pattern plot.
///
/// Pieces are placed in columns across the paper width (simple shelf packing, tallest first).
/// This is not a marker — it is for plotting pattern card / checking patterns.
pub fn pattern_sheet(style: &Style, size_indices: &[usize], opt: &PatternPlotOptions) -> Sheet {
    struct Item {
        bbox: BBox,
        geo: PieceGeometry,
        label: String,
    }
    let mut items: Vec<Item> = Vec::new();
    for p in &style.pieces {
        for &s in size_indices {
            let Some(geo) = p.sizes.get(s) else { continue };
            let geo = align_to_grain(geo);
            let qty = p.quantity.get(s).copied().unwrap_or(1);
            items.push(Item {
                bbox: geo.bbox(),
                geo,
                label: format!(
                    "{} | {} | {} | x{}",
                    style.name, p.name, style.sizes[s].name, qty
                ),
            });
        }
    }
    items.sort_by(|a, b| b.bbox.height().total_cmp(&a.bbox.height()));

    let mut sheet = Sheet {
        width: opt.paper_width,
        shapes: Vec::new(),
    };
    let (mut col_x, mut col_w, mut y) = (0.0_f64, 0.0_f64, 0.0_f64);
    for it in items {
        let (w, h) = (it.bbox.width(), it.bbox.height());
        if y > 0.0 && y + h > opt.paper_width {
            col_x += col_w + opt.gap;
            col_w = 0.0;
            y = 0.0;
        }
        let t = Transform {
            angle: 0.0,
            mirror_x: false,
            offset: Vec2::new(col_x, y) - it.bbox.min,
        };
        let geo = &it.geo;
        sheet.shapes.push(Shape::Polyline {
            pen: Pen::Cut,
            closed: true,
            points: t.apply_all(&geo.outline),
        });
        for l in &geo.internals {
            sheet.shapes.push(Shape::Polyline {
                pen: Pen::Internal,
                closed: false,
                points: t.apply_all(l),
            });
        }
        if let Some(g) = &geo.grain {
            let (a, b) = (t.apply(g.a), t.apply(g.b));
            sheet.shapes.push(Shape::Polyline {
                pen: Pen::Internal,
                closed: false,
                points: grain_with_arrows(a, b),
            });
            // label just above the grain line, starting at its first point
            sheet.shapes.push(Shape::Text {
                pos: a + Vec2::new(0.0, opt.text_height * 0.6),
                height: opt.text_height,
                text: it.label,
            });
        } else {
            let c = Vec2::new(col_x + w * 0.1, y + h / 2.0);
            sheet.shapes.push(Shape::Text {
                pos: c,
                height: opt.text_height,
                text: it.label,
            });
        }
        col_w = col_w.max(w);
        y += h + opt.gap;
    }
    sheet
}

/// Rotate a piece so its grain line runs exactly along X (the paper/fabric roll).
/// The smaller of the two possible turns is used, so pieces are never turned upside down
/// more than needed. Pieces without a grain line are returned unchanged.
pub fn align_to_grain(geo: &PieceGeometry) -> PieceGeometry {
    let Some(g) = &geo.grain else {
        return geo.clone();
    };
    let mut a = (g.b - g.a).angle();
    // reduce to (-90°, 90°]: grain direction has no sign
    while a > std::f64::consts::FRAC_PI_2 {
        a -= std::f64::consts::PI;
    }
    while a <= -std::f64::consts::FRAC_PI_2 {
        a += std::f64::consts::PI;
    }
    let t = Transform {
        angle: -a,
        mirror_x: false,
        offset: Vec2::new(0.0, 0.0),
    };
    PieceGeometry {
        outline: t.apply_all(&geo.outline),
        grain: Some(GrainLine {
            a: t.apply(g.a),
            b: t.apply(g.b),
        }),
        internals: geo.internals.iter().map(|l| t.apply_all(l)).collect(),
    }
}

/// Grain line drawn as one polyline with an arrow head at both ends.
fn grain_with_arrows(a: Vec2, b: Vec2) -> Vec<Vec2> {
    let d = b - a;
    let len = d.length();
    if len < 1e-6 {
        return vec![a, b];
    }
    let u = d * (1.0 / len);
    let n = Vec2::new(-u.y, u.x);
    let s = 8.0_f64.min(len / 4.0);
    vec![
        a + u * s + n * (s * 0.5),
        a,
        a + u * s - n * (s * 0.5),
        a,
        b,
        b - u * s + n * (s * 0.5),
        b,
        b - u * s - n * (s * 0.5),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grain_becomes_horizontal_without_flipping() {
        // grain pointing 30° below the X axis, going left (210°)
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(
            -100.0 * 30f64.to_radians().cos(),
            -100.0 * 30f64.to_radians().sin(),
        );
        let geo = PieceGeometry {
            outline: vec![a, Vec2::new(10.0, 0.0), Vec2::new(10.0, 10.0)],
            grain: Some(GrainLine { a, b }),
            internals: vec![],
        };
        let out = align_to_grain(&geo);
        let g = out.grain.unwrap();
        assert!((g.a.y - g.b.y).abs() < 1e-9, "grain not horizontal: {g:?}");
        // area is preserved by rotation
        assert!((gcad_geom::area(&out.outline) - gcad_geom::area(&geo.outline)).abs() < 1e-9);
    }
}
