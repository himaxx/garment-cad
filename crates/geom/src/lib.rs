//! Basic 2D geometry. All lengths are millimetres (f64).

use serde::{Deserialize, Serialize};
use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn length(self) -> f64 {
        self.x.hypot(self.y)
    }
    pub fn distance(self, o: Vec2) -> f64 {
        (self - o).length()
    }
    /// Rotate counter-clockwise by `angle` radians about the origin.
    pub fn rotated(self, angle: f64) -> Vec2 {
        let (s, c) = angle.sin_cos();
        Vec2::new(self.x * c - self.y * s, self.x * s + self.y * c)
    }
    pub fn angle(self) -> f64 {
        self.y.atan2(self.x)
    }
}

impl Add for Vec2 {
    type Output = Vec2;
    fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }
}
impl Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }
}
impl Mul<f64> for Vec2 {
    type Output = Vec2;
    fn mul(self, k: f64) -> Vec2 {
        Vec2::new(self.x * k, self.y * k)
    }
}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BBox {
    pub min: Vec2,
    pub max: Vec2,
}

impl BBox {
    pub fn empty() -> Self {
        BBox {
            min: Vec2::new(f64::INFINITY, f64::INFINITY),
            max: Vec2::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }
    pub fn of(points: &[Vec2]) -> Self {
        let mut b = BBox::empty();
        for p in points {
            b.include(*p);
        }
        b
    }
    pub fn include(&mut self, p: Vec2) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
    }
    pub fn union(&mut self, o: &BBox) {
        if o.is_empty() {
            return;
        }
        self.include(o.min);
        self.include(o.max);
    }
    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x
    }
    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }
    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }
}

/// Signed area of a closed polygon (positive = counter-clockwise), shoelace formula.
pub fn signed_area(poly: &[Vec2]) -> f64 {
    if poly.len() < 3 {
        return 0.0;
    }
    let mut a = 0.0;
    for i in 0..poly.len() {
        let p = poly[i];
        let q = poly[(i + 1) % poly.len()];
        a += p.x * q.y - q.x * p.y;
    }
    a / 2.0
}

pub fn area(poly: &[Vec2]) -> f64 {
    signed_area(poly).abs()
}

/// Length of a polyline; when `closed`, includes the closing segment.
pub fn perimeter(poly: &[Vec2], closed: bool) -> f64 {
    let mut l: f64 = poly.windows(2).map(|w| w[0].distance(w[1])).sum();
    if closed && poly.len() > 2 {
        l += poly[poly.len() - 1].distance(poly[0]);
    }
    l
}

/// Affine transform limited to rotation + translation (+ optional mirror in X).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub angle: f64,
    pub mirror_x: bool,
    pub offset: Vec2,
}

impl Transform {
    pub const IDENTITY: Transform = Transform {
        angle: 0.0,
        mirror_x: false,
        offset: Vec2::new(0.0, 0.0),
    };
    pub fn apply(&self, p: Vec2) -> Vec2 {
        let p = if self.mirror_x {
            Vec2::new(-p.x, p.y)
        } else {
            p
        };
        p.rotated(self.angle) + self.offset
    }
    pub fn apply_all(&self, pts: &[Vec2]) -> Vec<Vec2> {
        pts.iter().map(|p| self.apply(*p)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(s: f64) -> Vec<Vec2> {
        vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(s, 0.0),
            Vec2::new(s, s),
            Vec2::new(0.0, s),
        ]
    }

    #[test]
    fn area_and_perimeter_of_square() {
        let sq = square(10.0);
        assert_eq!(signed_area(&sq), 100.0);
        assert_eq!(perimeter(&sq, true), 40.0);
        assert_eq!(perimeter(&sq, false), 30.0);
    }

    #[test]
    fn clockwise_area_is_negative() {
        let mut sq = square(2.0);
        sq.reverse();
        assert_eq!(signed_area(&sq), -4.0);
        assert_eq!(area(&sq), 4.0);
    }

    #[test]
    fn bbox_of_points() {
        let b = BBox::of(&[Vec2::new(1.0, 5.0), Vec2::new(-2.0, 3.0)]);
        assert_eq!(b.width(), 3.0);
        assert_eq!(b.height(), 2.0);
    }

    #[test]
    fn rotate_quarter_turn() {
        let p = Vec2::new(1.0, 0.0).rotated(std::f64::consts::FRAC_PI_2);
        assert!((p.x).abs() < 1e-12 && (p.y - 1.0).abs() < 1e-12);
    }
}
