/// Generate test DWG files with varying entity counts for performance testing
/// Run: cargo run --release --bin test_data_generator -- --count 5000 --output test_5k.dwg

use clap::Parser;
use codec::entities::{Arc as AcadArc, Circle, Line, LwPolyline, LwVertex};
use codec::types::{Vector2, Vector3};
use codec::{CadDocument, DxfWriter};
use std::io::Cursor;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    /// Number of entities to generate
    #[arg(short, long, default_value = "5000")]
    count: usize,

    /// Output DXF file path
    #[arg(short, long, default_value = "test_entities.dxf")]
    output: PathBuf,

    /// Entity mix: "mixed" (40% lines, 30% circles, 20% arcs, 10% polylines)
    /// or "lines", "circles", "arcs", "polylines"
    #[arg(short, long, default_value = "mixed")]
    entity_type: String,
}

fn main() {
    let args = Args::parse();

    println!("Generating {} entities ({})...", args.count, args.entity_type);

    let mut doc = CadDocument::new();

    match args.entity_type.as_str() {
        "lines" => generate_lines(&mut doc, args.count),
        "circles" => generate_circles(&mut doc, args.count),
        "arcs" => generate_arcs(&mut doc, args.count),
        "polylines" => generate_polylines(&mut doc, args.count),
        "mixed" => {
            let n_lines = (args.count * 40) / 100;
            let n_circles = (args.count * 30) / 100;
            let n_arcs = (args.count * 20) / 100;
            let n_polylines = args.count - n_lines - n_circles - n_arcs;

            generate_lines(&mut doc, n_lines);
            generate_circles(&mut doc, n_circles);
            generate_arcs(&mut doc, n_arcs);
            generate_polylines(&mut doc, n_polylines);
        }
        _ => panic!("Unknown entity type: {}", args.entity_type),
    }

    // Write to DXF
    let mut buffer = Cursor::new(Vec::new());
    let mut writer = DxfWriter::new(&mut buffer).expect("Failed to create DXF writer");
    writer.write_document(&doc).expect("Failed to write DXF");

    let bytes = buffer.into_inner();
    std::fs::write(&args.output, &bytes).expect("Failed to write file");

    println!(
        "✓ Generated {} entities -> {}",
        args.count,
        args.output.display()
    );
}

fn generate_lines(doc: &mut CadDocument, count: usize) {
    for i in 0..count {
        let x = (i % 100) as f64 * 20.0;
        let y = (i / 100) as f64 * 20.0;
        let mut line = Line::new();
        line.start = Vector3::new(x, y, 0.0);
        line.end = Vector3::new(x + 15.0, y + 15.0, 0.0);
        line.color = (i % 256) as u8;
        doc.add_entity(line);
    }
}

fn generate_circles(doc: &mut CadDocument, count: usize) {
    for i in 0..count {
        let x = (i % 100) as f64 * 20.0;
        let y = (i / 100) as f64 * 20.0;
        let mut circle = Circle::new();
        circle.center = Vector3::new(x, y, 0.0);
        circle.radius = 5.0 + (i % 10) as f64;
        circle.color = (i % 256) as u8;
        doc.add_entity(circle);
    }
}

fn generate_arcs(doc: &mut CadDocument, count: usize) {
    for i in 0..count {
        let x = (i % 100) as f64 * 20.0;
        let y = (i / 100) as f64 * 20.0;
        let mut arc = AcadArc::new();
        arc.center = Vector3::new(x, y, 0.0);
        arc.radius = 8.0 + (i % 10) as f64;
        arc.start_angle = 0.0;
        arc.end_angle = std::f64::consts::PI / 2.0 * ((i % 4) as f64 + 1.0);
        arc.color = (i % 256) as u8;
        doc.add_entity(arc);
    }
}

fn generate_polylines(doc: &mut CadDocument, count: usize) {
    for i in 0..count {
        let base_x = (i % 50) as f64 * 40.0;
        let base_y = (i / 50) as f64 * 40.0;
        let mut poly = LwPolyline::new();

        // Create a zigzag pattern
        for j in 0..10 {
            let x = base_x + (j as f64 * 4.0);
            let y = base_y + if j % 2 == 0 { 0.0 } else { 5.0 };
            poly.vertices.push(LwVertex {
                x: Vector2::new(x, y),
                start_width: 0.0,
                end_width: 0.0,
                bulge: 0.0,
            });
        }
        poly.color = (i % 256) as u8;
        doc.add_entity(poly);
    }
}
