use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{Read, Result as IoResult};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Peak {
    row: usize,
    col: usize,
    elevation: i16,
    prominence: i16,
    col_row: Option<usize>,
    col_col: Option<usize>,
    col_elevation: Option<i16>,
}

impl Peak {
    fn new(row: usize, col: usize, elevation: i16) -> Self {
        Peak {
            row,
            col,
            elevation,
            prominence: 0,
            col_row: None,
            col_col: None,
            col_elevation: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Cell {
    elevation: i16,
    row: usize,
    col: usize,
    index: usize,
}

impl Ord for Cell {
    fn cmp(&self, other: &Self) -> Ordering {
        // FIXED: Use ascending order for proper drainage basin formation
        self.elevation.cmp(&other.elevation)
    }
}

impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<u8>,
    peak_elevation: Vec<i16>,
    key_saddle_elevation: Vec<i16>,
    drains_to_boundary: Vec<bool>,
    peak_index: Vec<Option<usize>>,
    saddle_index: Vec<Option<usize>>,
    grid_width: usize,
    grid_height: usize,
}

impl UnionFind {
    fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        UnionFind {
            parent: (0..size).collect(),
            rank: vec![0; size],
            peak_elevation: vec![i16::MIN; size],
            key_saddle_elevation: vec![i16::MIN; size],
            drains_to_boundary: vec![false; size],
            peak_index: vec![None; size],
            saddle_index: vec![None; size],
            grid_width: width,
            grid_height: height,
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, x: usize, y: usize, current_elevation: i16, current_index: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x == root_y {
            return;
        }

        // Merge components and track the saddle where they connect
        self.merge_components(root_x, root_y, current_elevation, current_index);
    }

    fn merge_components(&mut self, root_x: usize, root_y: usize, current_elevation: i16, current_index: usize) {
        // Determine which component should be the parent
        let (parent_root, child_root) = if self.rank[root_x] >= self.rank[root_y] {
            (root_x, root_y)
        } else {
            (root_y, root_x)
        };

        // Update drainage to boundary status
        self.drains_to_boundary[parent_root] = 
            self.drains_to_boundary[parent_root] || self.drains_to_boundary[child_root];

        // Update peak information - keep the higher peak
        if self.peak_elevation[child_root] > self.peak_elevation[parent_root] {
            self.peak_elevation[parent_root] = self.peak_elevation[child_root];
            self.peak_index[parent_root] = self.peak_index[child_root];
        }

        // FIXED: Update key saddle - this is where two drainage basins meet
        // The key saddle is the highest point we must cross to get to higher terrain
        if current_elevation > self.key_saddle_elevation[parent_root] {
            self.key_saddle_elevation[parent_root] = current_elevation;
            self.saddle_index[parent_root] = Some(current_index);
        }
        
        // Also consider the previous saddle of the child component
        if self.key_saddle_elevation[child_root] > self.key_saddle_elevation[parent_root] {
            self.key_saddle_elevation[parent_root] = self.key_saddle_elevation[child_root];
            self.saddle_index[parent_root] = self.saddle_index[child_root];
        }

        // Perform the union
        self.parent[child_root] = parent_root;
        if self.rank[root_x] == self.rank[root_y] {
            self.rank[parent_root] += 1;
        }
    }

    fn mark_as_peak(&mut self, index: usize, elevation: i16) {
        let root = self.find(index);
        if elevation > self.peak_elevation[root] {
            self.peak_elevation[root] = elevation;
            self.peak_index[root] = Some(index);
        }
    }

    fn mark_boundary(&mut self, index: usize) {
        let root = self.find(index);
        self.drains_to_boundary[root] = true;
    }

    fn get_coords_from_index(&self, index: usize) -> (usize, usize) {
        (index / self.grid_width, index % self.grid_width)
    }

    fn collect_peaks(&mut self, min_prominence: i16) -> Vec<Peak> {
        let mut peak_map = HashMap::new();
        let mut component_count = 0;
        let mut valid_peak_count = 0;

        for i in 0..self.parent.len() {
            let root = self.find(i);
            if let Some(peak_idx) = self.peak_index[root] {
                // Only process each component once
                if peak_map.contains_key(&peak_idx) {
                    continue;
                }
                
                component_count += 1;
                
                let peak_elev = self.peak_elevation[root];
                
                // FIXED: Proper prominence calculation
                let prominence = if self.drains_to_boundary[root] {
                    // Peak drains to boundary (sea level = 0)
                    peak_elev
                } else {
                    // Peak is enclosed by higher terrain
                    let saddle_elev = self.key_saddle_elevation[root];
                    if saddle_elev > i16::MIN {
                        peak_elev - saddle_elev
                    } else {
                        0 // Shouldn't happen for real peaks
                    }
                };

                if prominence >= min_prominence && prominence > 0 {
                    valid_peak_count += 1;
                    let (peak_row, peak_col) = self.get_coords_from_index(peak_idx);
                    
                    let (col_row, col_col, col_elevation) = if self.drains_to_boundary[root] {
                        // No col for peaks draining to boundary
                        (None, None, None)
                    } else if let Some(saddle_idx) = self.saddle_index[root] {
                        let (r, c) = self.get_coords_from_index(saddle_idx);
                        (Some(r), Some(c), Some(self.key_saddle_elevation[root]))
                    } else {
                        (None, None, None)
                    };

                    peak_map.insert(peak_idx, Peak {
                        row: peak_row,
                        col: peak_col,
                        elevation: peak_elev,
                        prominence,
                        col_row,
                        col_col,
                        col_elevation,
                    });
                }
            }
        }

        println!("Found {} components with peaks, {} valid peaks with prominence >= {}", 
                 component_count, valid_peak_count, min_prominence);

        let mut peaks: Vec<Peak> = peak_map.into_values().collect();
        peaks.sort_by(|a, b| b.prominence.cmp(&a.prominence));
        peaks
    }
}

struct TopographicProminence {
    grid: Vec<Vec<i16>>,
    width: usize,
    height: usize,
}

impl TopographicProminence {
    fn new(grid: Vec<Vec<i16>>) -> Self {
        let height = grid.len();
        let width = if height > 0 { grid[0].len() } else { 0 };

        TopographicProminence {
            grid,
            width,
            height,
        }
    }

    // FIXED: Made dimensions configurable and added proper error handling
    fn load_from_binary(filename: &str) -> IoResult<Self> {
        println!("Loading binary file: {}", filename);
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Try to detect dimensions from file size
        let total_cells = buffer.len() / 2; // 2 bytes per i16
        
        // Common DEM dimensions - adjust as needed
        let (width, height) = if total_cells == 6000 * 4800 {
            (6000, 4800)
        } else if total_cells == 1200 * 1200 {
            (1200, 1200)
        } else {
            // Default assumption or calculate square
            let side = (total_cells as f64).sqrt() as usize;
            if side * side == total_cells {
                (side, side)
            } else {
                eprintln!("Warning: Cannot determine grid dimensions from file size");
                eprintln!("File has {} bytes ({} cells), using default 6000x4800", 
                         buffer.len(), total_cells);
                (6000, 4800)
            }
        };

        let mut grid = vec![vec![0i16; width]; height];

        for (i, chunk) in buffer.chunks_exact(2).enumerate() {
            if i >= width * height {
                break;
            }
            let value = i16::from_le_bytes([chunk[0], chunk[1]]).max(0);
            let row = i / width;
            let col = i % width;
            grid[row][col] = value;
        }

        println!("Grid loaded: {} x {} ({} cells)", width, height, width * height);
        Ok(TopographicProminence::new(grid))
    }

    fn get_neighbor_indices(&self, row: usize, col: usize) -> Vec<usize> {
        let mut neighbors = Vec::with_capacity(8);
        let row_i32 = row as i32;
        let col_i32 = col as i32;

        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue;
                }

                let nr = row_i32 + dr;
                let nc = col_i32 + dc;

                if nr >= 0 && nr < self.height as i32 && nc >= 0 && nc < self.width as i32 {
                    neighbors.push((nr as usize) * self.width + (nc as usize));
                }
            }
        }
        neighbors
    }

    // FIXED: Better peak detection that handles flat summits
    fn is_peak(&self, row: usize, col: usize) -> bool {
        let elevation = self.grid[row][col];
        if elevation <= 0 {
            return false;
        }

        let mut has_lower_neighbor = false;
        
        for dr in -1..=1i32 {
            for dc in -1..=1i32 {
                if dr == 0 && dc == 0 {
                    continue;
                }

                let nr = row as i32 + dr;
                let nc = col as i32 + dc;

                if nr >= 0 && nr < self.height as i32 && nc >= 0 && nc < self.width as i32 {
                    let neighbor_elev = self.grid[nr as usize][nc as usize];
                    if neighbor_elev > elevation {
                        return false; // Has higher neighbor, not a peak
                    }
                    if neighbor_elev < elevation {
                        has_lower_neighbor = true;
                    }
                }
            }
        }
        
        // Must have at least one lower neighbor to be considered a peak
        has_lower_neighbor
    }

    fn is_on_boundary(&self, row: usize, col: usize) -> bool {
        row == 0 || row == self.height - 1 || col == 0 || col == self.width - 1
    }

    fn calculate_prominence_union_find(&self, min_elevation: i16, min_prominence: i16) -> Vec<Peak> {
        println!("Starting union-find prominence calculation...");
        let start_time = std::time::Instant::now();

        let mut uf = UnionFind::new(self.width, self.height);
        let mut all_cells = Vec::new();
        let mut processed = vec![false; self.width * self.height];

        // Collect all valid cells
        for row in 0..self.height {
            for col in 0..self.width {
                let elevation = self.grid[row][col];
                if elevation < min_elevation {
                    continue;
                }

                let index = row * self.width + col;
                
                // Mark boundary cells
                if self.is_on_boundary(row, col) {
                    uf.mark_boundary(index);
                }

                // Mark peaks
                if self.is_peak(row, col) {
                    uf.mark_as_peak(index, elevation);
                }

                all_cells.push(Cell {
                    elevation,
                    row,
                    col,
                    index,
                });
            }
        }

        // FIXED: Sort in ASCENDING order to build drainage basins properly
        all_cells.sort_by(|a, b| a.elevation.cmp(&b.elevation));

        println!("Processing {} cells in ascending elevation order...", all_cells.len());

        // Process cells from lowest to highest
        for (i, cell) in all_cells.iter().enumerate() {
            processed[cell.index] = true;
            
            if i % 1_000_000 == 0 && i > 0 {
                println!("Processed {}/{} cells ({:.1}%)", 
                        i, all_cells.len(), (i as f64 / all_cells.len() as f64) * 100.0);
            }

            // FIXED: Connect to lower or equal neighbors that have been processed
            for neighbor_idx in self.get_neighbor_indices(cell.row, cell.col) {
                if processed[neighbor_idx] {
                    let neighbor_row = neighbor_idx / self.width;
                    let neighbor_col = neighbor_idx % self.width;
                    let neighbor_elev = self.grid[neighbor_row][neighbor_col];
                    
                    if neighbor_elev <= cell.elevation {
                        uf.union(cell.index, neighbor_idx, cell.elevation, cell.index);
                    }
                }
            }
        }

        println!("Union-find completed in {:.2?}", start_time.elapsed());
        println!("Collecting results...");

        let peaks = uf.collect_peaks(min_prominence);
        println!("Total calculation time: {:.2?}", start_time.elapsed());

        peaks
    }
}

fn main() -> IoResult<()> {
    println!("Topographic Prominence Calculator (Corrected Union-Find Algorithm)");
    
    let args: Vec<String> = std::env::args().collect();
    let filename = args.get(1).map(|s| s.as_str()).unwrap_or("W100N40.bin");

    if !std::path::Path::new(filename).exists() {
        eprintln!("Error: File '{}' not found", filename);
        eprintln!("Usage: {} [filename.bin]", args[0]);
        std::process::exit(1);
    }

    let load_start = std::time::Instant::now();
    let topo = TopographicProminence::load_from_binary(filename)?;
    println!("Data loaded in {:.2?}", load_start.elapsed());

    let peaks = topo.calculate_prominence_union_find(100, 100);

    // Display results in the required format
    println!("\nPeaks by prominence:");
    println!("  prom    row    col   elev   crow   ccol  celev");
    println!("--------------------------------------------------");

    for peak in peaks.iter().take(100) {
        let (crow_str, ccol_str, celev_str) = match (peak.col_row, peak.col_col, peak.col_elevation) {
            (Some(crow), Some(ccol), Some(celev)) => {
                (format!("{:6}", crow), format!("{:6}", ccol), format!("{:6}", celev))
            }
            _ => ("    NA".to_string(), "    NA".to_string(), "    NA".to_string())
        };

        println!("{:6} {:6} {:6} {:6} {} {} {}",
                 peak.prominence, peak.row, peak.col, peak.elevation,
                 crow_str, ccol_str, celev_str);
    }

    println!("\nFound {} peaks with prominence >= {}", peaks.len(), 100);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_grid() -> Vec<Vec<i16>> {
        // Simple test case with clear peak hierarchy
        vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 2, 1, 2, 0],
            vec![0, 1, 5, 1, 0],
            vec![0, 2, 1, 2, 0],
            vec![0, 0, 0, 0, 0],
        ]
    }

    #[test]
    fn test_corrected_prominence_calculation() {
        let grid = create_test_grid();
        let topo = TopographicProminence::new(grid);
        let peaks = topo.calculate_prominence_union_find(1, 1);
        
        // Central peak should have prominence = 5 (drains to boundary at 0)
        let central_peak = peaks.iter().find(|p| p.elevation == 5).unwrap();
        assert_eq!(central_peak.prominence, 5);
        
        // Corner peaks should have lower prominence
        let corner_peaks: Vec<_> = peaks.iter().filter(|p| p.elevation == 2).collect();
        assert!(!corner_peaks.is_empty());
        for peak in corner_peaks {
            assert!(peak.prominence < 5);
        }
    }

    #[test]
    fn test_boundary_detection() {
        let grid = create_test_grid();
        let topo = TopographicProminence::new(grid);
        
        assert!(topo.is_on_boundary(0, 0));
        assert!(topo.is_on_boundary(4, 4));
        assert!(!topo.is_on_boundary(2, 2));
    }

    #[test]
    fn test_peak_detection() {
        let grid = create_test_grid();
        let topo = TopographicProminence::new(grid);
        
        assert!(topo.is_peak(2, 2)); // Central peak
        assert!(topo.is_peak(1, 1)); // Corner peak
        assert!(!topo.is_peak(1, 2)); // Saddle point
    }
}