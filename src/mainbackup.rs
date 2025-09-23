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
        other.prominence.cmp(&self.prominence) // Max-heap: higher prominence first
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

// Union-Find data structure
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
            highest_point: vec![Point { elevation: 0, x: 0, y: 0, index: 0 }; size],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize, col_point: Point, peaks: &mut BinaryHeap<Peak>, peaks_set: &Vec<bool>) {
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

    fn merge(&mut self, smaller: usize, larger: usize, col_point: Point, peaks: &mut BinaryHeap<Peak>, peaks_set: &Vec<bool>) {
        self.parent[smaller] = larger;

        let smaller_peak = self.highest_point[smaller];
        let larger_peak = self.highest_point[larger];

        // Only compute prominence if smaller set's highest point is a peak and merges with a higher peak
        if peaks_set[smaller_peak.index] && smaller_peak.elevation > 0 && smaller_peak.elevation <= larger_peak.elevation {
            let prominence = smaller_peak.elevation - col_point.elevation;
            if prominence > 0 { // Only add peaks with positive prominence
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

// Read grid from CSV file
fn read_csv_grid(filename: &str) -> io::Result<Vec<Vec<i32>>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut grid = vec![];
    for line in std::io::read_to_string(reader)?.lines() {
        let row: Vec<i32> = line
            .split(',')
            .map(|s| s.trim().parse().expect("Invalid number in CSV"))
            .collect();
        if !row.is_empty() {
            grid.push(row);
        }
    }
    if grid.is_empty() || grid.iter().any(|row| row.len() != grid[0].len()) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid CSV grid"));
    }
    Ok(grid)
}

// Read grid from binary file
fn read_bin_grid(filename: &str) -> io::Result<Vec<Vec<i32>>> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 4];

    reader.read_exact(&mut buffer)?;
    let rows = i32::from_be_bytes(buffer) as usize;
    reader.read_exact(&mut buffer)?;
    let cols = i32::from_be_bytes(buffer) as usize;

    if rows == 0 || cols == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid grid dimensions"));
    }

    let mut grid = vec![vec![0i32; cols]; rows];
    for x in 0..rows {
        for y in 0..cols {
            reader.read_exact(&mut buffer)?;
            grid[x][y] = i32::from_be_bytes(buffer);
            if grid[x][y] < 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Negative elevation in binary file"));
            }
        }
    }
    Ok(grid)
}

// Compute prominence using Union-Find
fn compute_prominence(grid: Vec<Vec<i32>>) -> Vec<Peak> {
    let rows = grid.len();
    if rows == 0 {
        return vec![];
    }
    let cols = grid[0].len();
    let total_points = rows * cols;

    // Step 1: Identify peaks and collect points
    let mut points = vec![];
    let mut peaks_set = vec![false; total_points];
    for x in 0..rows {
        for y in 0..cols {
            let index = x * cols + y;
            points.push(Point {
                elevation: grid[x][y],
                x,
                y,
                index,
            });

            let mut is_peak = true;
            for &(dx, dy) in &[
                (-1, -1), (-1, 0), (-1, 1),
                (0, -1),           (0, 1),
                (1, -1),  (1, 0),  (1, 1),
            ] {
                let nx = x as i32 + dx;
                let ny = y as i32 + dy;
                if nx >= 0 && nx < rows as i32 && ny >= 0 && ny < cols as i32 {
                    if grid[nx as usize][ny as usize] >= grid[x][y] {
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

    // Step 2: Sort points by elevation
    points.sort();

    // Step 3: Initialize Union-Find
    let mut uf = UnionFind::new(total_points);
    let mut result_peaks = BinaryHeap::new();

    // Step 4: Process points in descending elevation order
    for &point in points.iter() {
        let index = point.index;
        uf.highest_point[index] = point;

        // Add the highest peak with col as NA NA NA
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

        // Collect neighbors to union
        let mut neighbors_to_union = vec![];
        for &(dx, dy) in &[
            (-1, -1), (-1, 0), (-1, 1),
            (0, -1),           (0, 1),
            (1, -1),  (1, 0),  (1, 1),
        ] {
            let nx = point.x as i32 + dx;
            let ny = point.y as i32 + dy;
            if nx >= 0 && nx < rows as i32 && ny >= 0 && ny < cols as i32 {
                let neighbor_index = (nx as usize) * cols + ny as usize;
                let neighbor_root = uf.find(neighbor_index);
                if uf.highest_point[neighbor_root].elevation >= point.elevation {
                    neighbors_to_union.push(neighbor_index);
                }
            }
        }

        // Perform unions
        for neighbor_index in neighbors_to_union {
            uf.union(index, neighbor_index, point, &mut result_peaks, &peaks_set);
        }
    }

    // Step 5: Collect top 100 peaks in descending order
    let mut output = vec![];
    let mut count = 0;
    while let Some(peak) = result_peaks.pop() {
        if count >= 100 {
            break;
        }
        output.push(peak);
        count += 1;
    }

    output // Already in descending order due to BinaryHeap
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: cargo run -- <filename>");
        std::process::exit(1);
    }
    let filename = &args[1];

    let grid = match Path::new(filename).extension().and_then(|ext| ext.to_str()) {
        Some("csv") => read_csv_grid(filename)?,
        Some("bin") => read_bin_grid(filename)?,
        _ => {
            eprintln!("Unsupported file extension. Use .csv or .bin");
            std::process::exit(1);
        }
    };

    let mut peaks = compute_prominence(grid);

    // Sort peaks by descending prominence
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
