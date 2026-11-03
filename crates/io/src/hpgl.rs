//! HPGL / PLT writer for inkjet and cutting plotters.
//!
//! Plotter units: 40 per mm (0.025 mm). Calibration factors scale X and Y so that a
//! plotted 1000 mm line measures exactly 1000 mm on the paper.

use crate::sheet::{Pen, Shape, Sheet};
use std::fmt::Write;

pub const UNITS_PER_MM: f64 = 40.0;

#[derive(Debug, Clone)]
pub struct HpglOptions {
    /// Measured-vs-expected correction along the roll (X), e.g. 1000 / 998.5.
    pub scale_x: f64,
    /// Correction across the paper (Y).
    pub scale_y: f64,
    /// Write text labels (LB). Some cutters reject text; turn off for them.
    pub text: bool,
    /// Map each pen to a plotter pen number.
    pub pen_cut: u8,
    pub pen_internal: u8,
    pub pen_text: u8,
}

impl Default for HpglOptions {
    fn default() -> Self {
        Self {
            scale_x: 1.0,
            scale_y: 1.0,
            text: true,
            pen_cut: 1,
            pen_internal: 1,
            pen_text: 1,
        }
    }
}

pub fn write(sheet: &Sheet, opt: &HpglOptions) -> String {
    let fx = UNITS_PER_MM * opt.scale_x;
    let fy = UNITS_PER_MM * opt.scale_y;
    let xy = |p: gcad_geom::Vec2| ((p.x * fx).round() as i64, (p.y * fy).round() as i64);

    let mut out = String::from("IN;");
    let mut current_pen = 0u8;
    let mut select = |out: &mut String, pen: u8| {
        if pen != current_pen {
            let _ = write!(out, "SP{pen};");
            current_pen = pen;
        }
    };

    for shape in &sheet.shapes {
        match shape {
            Shape::Polyline {
                pen,
                closed,
                points,
            } if points.len() >= 2 => {
                let pen_no = match pen {
                    Pen::Cut => opt.pen_cut,
                    Pen::Internal => opt.pen_internal,
                    Pen::Text => opt.pen_text,
                };
                select(&mut out, pen_no);
                let (x0, y0) = xy(points[0]);
                let _ = write!(out, "PU{x0},{y0};PD");
                let mut first = true;
                let rest = points[1..].iter().chain(closed.then_some(&points[0]));
                for p in rest {
                    let (x, y) = xy(*p);
                    if !first {
                        out.push(',');
                    }
                    let _ = write!(out, "{x},{y}");
                    first = false;
                }
                out.push(';');
            }
            Shape::Text { pos, height, text } if opt.text => {
                select(&mut out, opt.pen_text);
                let (x, y) = xy(*pos);
                // SI = character width, height in cm
                let h_cm = height / 10.0;
                let clean: String = text
                    .chars()
                    .filter(|c| c.is_ascii() && *c != '\u{3}' && *c != ';')
                    .collect();
                let _ = write!(
                    out,
                    "PU{x},{y};SI{:.3},{:.3};LB{}\u{3};",
                    h_cm * 0.6,
                    h_cm,
                    clean
                );
            }
            _ => {}
        }
    }
    out.push_str("PU;SP0;");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use gcad_geom::Vec2;

    fn square_sheet() -> Sheet {
        Sheet {
            width: 100.0,
            shapes: vec![Shape::Polyline {
                pen: Pen::Cut,
                closed: true,
                points: vec![
                    Vec2::new(0.0, 0.0),
                    Vec2::new(10.0, 0.0),
                    Vec2::new(10.0, 10.0),
                    Vec2::new(0.0, 10.0),
                ],
            }],
        }
    }

    #[test]
    fn closed_square_in_plotter_units() {
        let s = write(&square_sheet(), &HpglOptions::default());
        assert_eq!(s, "IN;SP1;PU0,0;PD400,0,400,400,0,400,0,0;PU;SP0;");
    }

    #[test]
    fn calibration_scales_axes() {
        let opt = HpglOptions {
            scale_x: 1.01,
            scale_y: 0.99,
            ..Default::default()
        };
        let s = write(&square_sheet(), &opt);
        assert!(s.contains("PD404,0,404,396,0,396,0,0;"), "{s}");
    }

    #[test]
    fn text_can_be_disabled() {
        let mut sheet = square_sheet();
        sheet.shapes.push(Shape::Text {
            pos: Vec2::new(1.0, 1.0),
            height: 10.0,
            text: "BACK;28".into(),
        });
        assert!(write(&sheet, &HpglOptions::default()).contains("LBBACK28\u{3};"));
        assert!(!write(
            &sheet,
            &HpglOptions {
                text: false,
                ..Default::default()
            }
        )
        .contains("LB"));
    }
}
