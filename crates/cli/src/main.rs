//! `gcad` — command-line tool for testing and batch work.
//!
//!   gcad info  GT_102.dgs [--json]
//!   gcad svg   GT_102.dgs -o out.svg [--size 28|all] [--paper-width 1600]
//!   gcad plt   GT_102.dgs -o out.plt [--size 28|all] [--paper-width 1600] [--scale-x 1.0] [--scale-y 1.0] [--no-text]

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use gcad_io::{hpgl, sheet, svg};
use gcad_model::Style;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "gcad", version, about = "Garment CAD command-line tool")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Show sizes, pieces, quantities and areas of a pattern file.
    Info {
        file: PathBuf,
        /// Print the full imported style as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Write an SVG preview of a 1:1 pattern plot.
    Svg {
        #[command(flatten)]
        plot: PlotArgs,
    },
    /// Write an HPGL/PLT file for the plotter.
    Plt {
        #[command(flatten)]
        plot: PlotArgs,
        /// Calibration factor along the roll (expected / measured).
        #[arg(long, default_value_t = 1.0)]
        scale_x: f64,
        /// Calibration factor across the paper.
        #[arg(long, default_value_t = 1.0)]
        scale_y: f64,
        /// Do not write text labels (for cutters that reject LB).
        #[arg(long)]
        no_text: bool,
    },
}

#[derive(Args)]
struct PlotArgs {
    file: PathBuf,
    #[arg(short, long)]
    output: PathBuf,
    /// Size name to plot, or "all". Default: base size.
    #[arg(long)]
    size: Option<String>,
    /// Usable paper width in mm.
    #[arg(long, default_value_t = 1600.0)]
    paper_width: f64,
}

fn load(path: &PathBuf) -> Result<Style> {
    let style =
        gcad_io::import_path(path).with_context(|| format!("reading {}", path.display()))?;
    for w in &style.warnings {
        eprintln!("warning: {w}");
    }
    Ok(style)
}

fn select_sizes(style: &Style, size: &Option<String>) -> Result<Vec<usize>> {
    match size.as_deref() {
        None => Ok(vec![style.base_size_index()]),
        Some(s) if s.eq_ignore_ascii_case("all") => Ok((0..style.sizes.len()).collect()),
        Some(s) => match style.size_index(s) {
            Some(i) => Ok(vec![i]),
            None => bail!(
                "size {s:?} not in this style (sizes: {})",
                style
                    .sizes
                    .iter()
                    .map(|s| s.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        },
    }
}

fn build_sheet(p: &PlotArgs) -> Result<(Style, sheet::Sheet)> {
    let style = load(&p.file)?;
    let sizes = select_sizes(&style, &p.size)?;
    let opt = sheet::PatternPlotOptions {
        paper_width: p.paper_width,
        ..Default::default()
    };
    let sh = sheet::pattern_sheet(&style, &sizes, &opt);
    Ok((style, sh))
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Info { file, json } => {
            let style = load(&file)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&style)?);
                return Ok(());
            }
            println!("Style: {}", style.name);
            let sizes: Vec<String> = style
                .sizes
                .iter()
                .map(|s| {
                    if s.base {
                        format!("[{}]", s.name)
                    } else {
                        s.name.clone()
                    }
                })
                .collect();
            println!("Sizes: {}   ([ ] = base size)", sizes.join(" "));
            let b = style.base_size_index();
            println!(
                "\n{:<16} {:>4} {:>10} {:>10} {:>12} {:>9}",
                "Piece", "Qty", "W mm", "H mm", "Area cm²", "Internals"
            );
            for p in &style.pieces {
                let g = &p.sizes[b];
                let bb = g.bbox();
                println!(
                    "{:<16} {:>4} {:>10.1} {:>10.1} {:>12.1} {:>9}",
                    p.name,
                    p.quantity[b],
                    bb.width(),
                    bb.height(),
                    g.area_mm2() / 100.0,
                    g.internals.len()
                );
            }
            println!("\nFabric area per garment (all pieces × quantity):");
            for (i, s) in style.sizes.iter().enumerate() {
                println!(
                    "  {:>6}: {:>9.1} cm²",
                    s.name,
                    style.garment_area_mm2(i) / 100.0
                );
            }
        }
        Cmd::Svg { plot } => {
            let (_, sh) = build_sheet(&plot)?;
            std::fs::write(&plot.output, svg::write(&sh))?;
            println!(
                "wrote {} (plot length {:.0} mm)",
                plot.output.display(),
                sh.length()
            );
        }
        Cmd::Plt {
            plot,
            scale_x,
            scale_y,
            no_text,
        } => {
            let (_, sh) = build_sheet(&plot)?;
            let opt = hpgl::HpglOptions {
                scale_x,
                scale_y,
                text: !no_text,
                ..Default::default()
            };
            std::fs::write(&plot.output, hpgl::write(&sh, &opt))?;
            println!(
                "wrote {} (plot length {:.0} mm on {:.0} mm paper)",
                plot.output.display(),
                sh.length(),
                plot.paper_width
            );
        }
    }
    Ok(())
}
