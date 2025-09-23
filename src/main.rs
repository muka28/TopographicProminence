use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::env;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

// Structure to represent a grid point with elevation and coordinates
#[derive(Clone, Copy, Eq, PartialEq)]
struct Point {
    elevation: i32,
    x: usize,
    y: usize,
    index: usize,
}

impl Ord for Point {
    fn cmp(&self, other: &Self) -> Ordering {
        other.elevation.cmp(&self.elevation) // Descending order
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Structure to represent a peak's prominence output
#[derive(Clone)]
struct Peak {
    prominence: i32,
    peak_x: usize,
    peak_y: usize,
    peak_elevation: i32,
    col_x: Option<usize>,
    col_y: Option<usize>,
    col_elevation: Option<i32>,
}

impl Ord for Peak {
    fn cmp(&self, other: &Self) -> Ordering {
        other.prominence.cmp(&self.prominence) // Max-heap
    }
}

impl PartialOrd for Peak {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Peak {}
impl PartialEq for Peak {
    fn eq(&self, other: &Self) -> bool {
        self.prominence == other.prominence
    }
}

// Union-Find structure
struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<i32>,
    highest_point: Vec<Point>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        UnionFind {
            parent: (0..size).collect(),
            rank: vec![0; size],
            highest_point: vec![
                Point {
                    elevation: i32::MIN,
                    x: 0,
                    y: 0,
                    index: 0
                };
                size
            ],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(
        &mut self,
        x: usize,
        y: usize,
        col_point: Point,
        peaks: &mut BinaryHeap<Peak>,
        peaks_set: &Vec<bool>,
    ) {
        let root_x = self.find(x);
        let root_y = self.find(y);
        if root_x == root_y {
            return;
        }

        if self.rank[root_x] < self.rank[root_y] {
            self.merge(root_x, root_y, col_point, peaks, peaks_set);
        } else if self.rank[root_x] > self.rank[root_y] {
            self.merge(root_y, root_x, col_point, peaks, peaks_set);
        } else {
            self.merge(root_y, root_x, col_point, peaks, peaks_set);
            self.rank[root_x] += 1;
        }
    }

    fn merge(
        &mut self,
        smaller: usize,
        larger: usize,
        col_point: Point,
        peaks: &mut BinaryHeap<Peak>,
        peaks_set: &Vec<bool>,
    ) {
        self.parent[smaller] = larger;

        let smaller_peak = self.highest_point[smaller];
        let larger_peak = self.highest_point[larger];

        // Only compute prominence if smaller set's highest point is a peak
        // and merges with a higher/equal peak
        if peaks_set[smaller_peak.index]
            && smaller_peak.elevation > 0
            && smaller_peak.elevation <= larger_peak.elevation
        {
            let prominence = smaller_peak.elevation - col_point.elevation;
            if prominence > 0 {
                peaks.push(Peak {
                    prominence,
                    peak_x: smaller_peak.x,
                    peak_y: smaller_peak.y,
                    peak_elevation: smaller_peak.elevation,
                    col_x: Some(col_point.x),
                    col_y: Some(col_point.y),
                    col_elevation: Some(col_point.elevation),
                });
            }
        }

        // Update the highest point in the merged set
        if self.highest_point[larger].elevation < self.highest_point[smaller].elevation {
            self.highest_point[larger] = self.highest_point[smaller];
        }
    }
}

// Read CSV into flat 1D grid
fn read_csv_grid(filename: &str) -> io::Result<(usize, usize, Vec<i32>)> {
    let content = std::fs::read_to_string(filename)?;
    let mut grid = Vec::new();
    let mut rows: usize = 0;
    let mut cols: usize = 0;

    for line in content.lines() {
        let row: Vec<i32> = line
            .split(',')
            .map(|s| s.trim().parse().expect("Invalid number in CSV"))
            .collect();
        if !row.is_empty() {
            if cols == 0 {
                cols = row.len();
            } else if cols != row.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Inconsistent row length in CSV",
                ));
            }
            grid.extend(row);
            rows += 1;
        }
    }

    if rows == 0 || cols == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Empty CSV grid",
        ));
    }

    let total: usize = rows
        .checked_mul(cols)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Grid size too large"))?;

    if total != grid.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "CSV grid size mismatch",
        ));
    }

    eprintln!("Read CSV grid: rows={}, cols={}, total={}", rows, cols, total);
    Ok((rows, cols, grid))
}

// Read binary grid into flat 1D grid
fn read_bin_grid(filename: &str) -> io::Result<(usize, usize, Vec<i32>)> {
    let rows: usize = 6000;
    let cols: usize = 4800;

    // Calculate total elements and check for overflow
    let total = rows
        .checked_mul(cols)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Grid size too large"))?;

    // Open the file and get its size
    let file = File::open(filename)?;
    let file_size = file.metadata()?.len();
    let expected_size = total as u64 * 2; // Each i16 is 2 bytes

    // Log file size information
    eprintln!(
        "File '{}': size = {} bytes, expected size = {} bytes (for {}x{} i16 grid)",
        filename, file_size, expected_size, rows, cols
    );

    // Validate file size
    if file_size != expected_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "File '{}' size mismatch: expected {} bytes (for {}x{} i16 grid), found {} bytes",
                filename, expected_size, rows, cols, file_size
            ),
        ));
    }

    // Log grid dimensions
    eprintln!("Reading BIN grid: rows={}, cols={}, total={}", rows, cols, total);

    let mut reader = BufReader::new(file);
    let mut grid = Vec::with_capacity(total);
    let mut buffer_16 = [0u8; 2];

    // Read elevation values
    for i in 0..total {
        reader.read_exact(&mut buffer_16)?;
        let val = i16::from_le_bytes(buffer_16) as i32; // Little-endian, i16 to i32
        
        // Clamp negative values to 0 (sea level) as per assignment requirements
        let elevation = if val < 0 { 0 } else { val };
        
        if i < 5 {
            eprintln!("Elevation[{}] = {} (original: {})", i, elevation, val);
        }
        
        grid.push(elevation);
    }

    // Verify grid size
    if grid.len() != total {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Binary grid size mismatch in file '{}': expected {} elements, read {}", filename, total, grid.len()),
        ));
    }

    eprintln!("Successfully read BIN grid: rows={}, cols={}, total={}", rows, cols, total);
    Ok((rows, cols, grid))
}// Compute prominence using Union-Find with flat grid
fn compute_prominence(rows: usize, cols: usize, grid: &[i32]) -> Vec<Peak> {
    let total_points = rows
        .checked_mul(cols)
        .expect("Grid size too large in compute_prominence");

    // Step 1: Identify peaks
    let mut points = Vec::with_capacity(total_points);
    let mut peaks_set = vec![false; total_points];

    for x in 0..rows {
        for y in 0..cols {
            let index = x * cols + y;
            let elevation = grid[index];
            points.push(Point {
                elevation,
                x,
                y,
                index,
            });

            let mut is_peak = true;
            for &(dx, dy) in &[
                (-1, -1),
                (-1, 0),
                (-1, 1),
                (0, -1),
                (0, 1),
                (1, -1),
                (1, 0),
                (1, 1),
            ] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && nx < rows as i32 && ny >= 0 && ny < cols as i32 {
                    if grid[(nx as usize) * cols + ny as usize] >= elevation {
                        is_peak = false;
                        break;
                    }
                }
            }
            if is_peak {
                peaks_set[index] = true;
            }
        }
    }

    // Step 2: Sort points descending
    points.sort();

    // Step 3: Union-Find
    let mut uf = UnionFind::new(total_points);
    let mut result_peaks = BinaryHeap::new();
    let mut activated = vec![false; total_points];

    // Step 4: Process points
    for &point in points.iter() {
        let index = point.index;
        uf.highest_point[index] = point;
        activated[index] = true;

        if peaks_set[index] && result_peaks.is_empty() {
            result_peaks.push(Peak {
                prominence: point.elevation,
                peak_x: point.x,
                peak_y: point.y,
                peak_elevation: point.elevation,
                col_x: None,
                col_y: None,
                col_elevation: None,
            });
        }

        // neighbors
        for &(dx, dy) in &[
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ] {
            let nx = point.x as i32 + dx;
            let ny = point.y as i32 + dy;
            if nx >= 0 && nx < rows as i32 && ny >= 0 && ny < cols as i32 {
                let neighbor_index = (nx as usize) * cols + ny as usize;
                if activated[neighbor_index] {
                    uf.union(index, neighbor_index, point, &mut result_peaks, &peaks_set);
                }
            }
        }
    }

    // Step 5: Top 100 peaks
    let mut output = Vec::new();
    for peak in result_peaks.into_sorted_vec().into_iter().take(100) {
        output.push(peak);
    }

    output
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <filename>");
        std::process::exit(1);
    }
    let filename = &args[1];

    // Read grid
    let (rows, cols, grid) = match Path::new(filename).extension().and_then(|ext| ext.to_str()) {
        Some("csv") => read_csv_grid(filename)?,
        Some("bin") => read_bin_grid(filename)?,
        _ => {
            eprintln!("Unsupported file extension. Use .csv or .bin");
            std::process::exit(1);
        }
    };

    // Compute prominence
    let mut peaks = compute_prominence(rows, cols, &grid);

    // Sort by descending prominence
    peaks.sort_by(|a, b| b.prominence.cmp(&a.prominence));

    println!("Peaks by prominence:");
    println!("  prom    row    col   elev   crow   ccol  celev");
    println!("--------------------------------------------------");
    for peak in peaks.iter() {
        let crow = peak.col_x.map_or("NA".to_string(), |x| format!("{:>4}", x));
        let ccol = peak.col_y.map_or("NA".to_string(), |y| format!("{:>4}", y));
        let celev = peak.col_elevation.map_or("NA".to_string(), |e| format!("{:>4}", e));
        println!(
            "{:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6}",
            peak.prominence,
            peak.peak_x,
            peak.peak_y,
            peak.peak_elevation,
            crow,
            ccol,
            celev
        );
    }

    Ok(())
}
