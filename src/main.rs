use topographic_prominence::*;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
   println!("Topographic Prominence Calculator (Clean Modular Implementation)");
    
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).unwrap_or("W100N40.bin");

    if !std::path::Path::new(filename).exists() {
        eprintln!("Error: File '{}' not found", filename);
        eprintln!("Usage: {} [filename.bin]", args[0]);
        std::process::exit(1);
    }

    let load_start = std::time::Instant::now();
    let grid = ElevationGrid::load_from_binary(filename)?;
    println!("Data loaded in {:.2?}", load_start.elapsed());

    let calculator = ProminenceCalculator::new(&grid);
    let peaks = calculator.calculate_prominence(100, 100)?;

    // Display results in the required format
    println!("\nPeaks by prominence:");
    println!("  prom    row    col   elev   crow   ccol  celev");
    println!("--------------------------------------------------");

    for peak in peaks.iter().take(100) {
        println!("{}", peak);
    }

    println!("\nFound {} peaks with prominence >= {}", peaks.len(), 100);
    Ok(())
}

