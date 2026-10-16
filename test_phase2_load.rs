// Simple test to verify Phase 2 DWG loading works
use std::path::Path;
use codec::{DwgReader, DxfReader};
use std::io::Cursor;

fn main() {
    let file_path = "/Users/voquochieu/Claude/target-game/OpenCADStudio/assets/files/GIẢI PHÁP AEE - PHIN.dwg";

    println!("=== Phase 2 DWG Loading Test ===");
    println!("File: {}", file_path);

    // Read file
    let bytes = match std::fs::read(file_path) {
        Ok(b) => {
            println!("✓ File read: {} bytes", b.len());
            b
        }
        Err(e) => {
            eprintln!("✗ Failed to read file: {}", e);
            return;
        }
    };

    // Detect extension
    let ext = Path::new(file_path)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());

    println!("Extension: {:?}", ext);

    // Parse based on format
    let doc = match ext.as_deref() {
        Some("dwg") => {
            println!("Parsing as DWG...");
            match DwgReader::from_stream(Cursor::new(bytes)) {
                Ok(mut reader) => {
                    match reader.read() {
                        Ok(doc) => {
                            println!("✓ DWG parsed successfully");
                            doc
                        }
                        Err(e) => {
                            eprintln!("✗ DWG read error: {}", e);
                            return;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("✗ DWG reader creation error: {}", e);
                    return;
                }
            }
        }
        Some("dxf") => {
            println!("Parsing as DXF...");
            match DxfReader::from_reader(Cursor::new(bytes)) {
                Ok(mut reader) => {
                    match reader.read() {
                        Ok(doc) => {
                            println!("✓ DXF parsed successfully");
                            doc
                        }
                        Err(e) => {
                            eprintln!("✗ DXF read error: {}", e);
                            return;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("✗ DXF reader creation error: {}", e);
                    return;
                }
            }
        }
        _ => {
            eprintln!("✗ Unsupported format");
            return;
        }
    };

    // Verify entities
    let entity_count = doc.entities().count();
    println!("✓ Entities loaded: {}", entity_count);

    // Sample first few entities
    let mut sample = 0;
    for _entity in doc.entities().take(10) {
        sample += 1;
    }
    println!("✓ Sampled {} entities - data is valid", sample);

    if entity_count > 0 {
        println!("\n=== SUCCESS ===");
        println!("DWG file loaded successfully with {} entities!", entity_count);
        println!("Phase 2 + viewport setup will display these entities.");
    } else {
        println!("\n=== WARNING ===");
        println!("File parsed but no entities found!");
    }
}
