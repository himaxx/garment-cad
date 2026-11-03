//! Document model.
//!
//! Phase 0/1 note: imported styles carry *evaluated* geometry per size (one outline per size),
//! which is what plotting and nesting need. The editable model (base size + grade rules,
//! see docs/02-architecture.md) will be added in Phase 3/4; imported geometry will remain
//! available as a read-only "graded geometry" view.

use gcad_geom::{area, perimeter, BBox, Vec2};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Size {
    pub name: String,
    /// True for the base (sample) size.
    pub base: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrainLine {
    pub a: Vec2,
    pub b: Vec2,
}

/// Geometry of one piece in one size.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PieceGeometry {
    /// Closed outline (cut line as stored by the source file), mm.
    pub outline: Vec<Vec2>,
    pub grain: Option<GrainLine>,
    /// Internal lines / marks (open polylines), mm.
    pub internals: Vec<Vec<Vec2>>,
}

impl PieceGeometry {
    pub fn bbox(&self) -> BBox {
        let mut b = BBox::of(&self.outline);
        for l in &self.internals {
            b.union(&BBox::of(l));
        }
        b
    }
    pub fn area_mm2(&self) -> f64 {
        area(&self.outline)
    }
    pub fn perimeter_mm(&self) -> f64 {
        perimeter(&self.outline, true)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Piece {
    pub name: String,
    /// Quantity per garment, per size (same order as `Style::sizes`).
    pub quantity: Vec<u32>,
    /// Geometry per size (same order as `Style::sizes`).
    pub sizes: Vec<PieceGeometry>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Style {
    pub name: String,
    pub sizes: Vec<Size>,
    pub pieces: Vec<Piece>,
    /// Non-fatal problems found while importing.
    pub warnings: Vec<String>,
}

impl Style {
    pub fn size_index(&self, name: &str) -> Option<usize> {
        self.sizes
            .iter()
            .position(|s| s.name.eq_ignore_ascii_case(name))
    }
    pub fn base_size_index(&self) -> usize {
        self.sizes.iter().position(|s| s.base).unwrap_or(0)
    }
    /// Total piece area for one garment of the given size (quantity included), mm².
    pub fn garment_area_mm2(&self, size: usize) -> f64 {
        self.pieces
            .iter()
            .filter_map(|p| {
                p.sizes
                    .get(size)
                    .map(|g| g.area_mm2() * *p.quantity.get(size).unwrap_or(&1) as f64)
            })
            .sum()
    }
}
