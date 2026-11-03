//! SVG preview of a sheet (1 SVG unit = 1 mm, Y flipped so the drawing matches the plot).

use crate::sheet::{Pen, Shape, Sheet};
use std::fmt::Write;

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn write(sheet: &Sheet) -> String {
    let b = sheet.bbox();
    let (w, h) = if b.is_empty() {
        (10.0, 10.0)
    } else {
        (b.max.x.max(1.0), sheet.width.max(b.max.y))
    };
    let m = 10.0;
    let mut out = String::new();
    let _ = write!(
        out,
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" width="{}mm" height="{}mm" style="background:#fff">"##,
        -m,
        -m,
        w + 2.0 * m,
        h + 2.0 * m,
        w + 2.0 * m,
        h + 2.0 * m
    );
    // paper edges
    let _ = write!(
        out,
        r##"<rect x="0" y="0" width="{w:.1}" height="{:.1}" fill="none" stroke="#bbb" stroke-dasharray="10 6"/>"##,
        sheet.width
    );
    let _ = write!(out, r#"<g transform="translate(0 {h:.2}) scale(1 -1)">"#);
    for s in &sheet.shapes {
        match s {
            Shape::Polyline {
                pen,
                closed,
                points,
            } => {
                let pts: Vec<String> = points
                    .iter()
                    .map(|p| format!("{:.2},{:.2}", p.x, p.y))
                    .collect();
                let (tag, color, width) = match pen {
                    Pen::Cut => (if *closed { "polygon" } else { "polyline" }, "#111", 0.6),
                    _ => (if *closed { "polygon" } else { "polyline" }, "#c0392b", 0.4),
                };
                let _ = write!(
                    out,
                    r##"<{tag} fill="none" stroke="{color}" stroke-width="{width}" points="{}"/>"##,
                    pts.join(" ")
                );
            }
            Shape::Text { pos, height, text } => {
                let _ = write!(
                    out,
                    r##"<text transform="translate({:.2} {:.2}) scale(1 -1)" font-family="sans-serif" font-size="{height}" fill="#1f4e9c">{}</text>"##,
                    pos.x,
                    pos.y,
                    esc(text)
                );
            }
        }
    }
    out.push_str("</g></svg>");
    out
}
