use std::fs::File;
use std::io::Read;
use crate::{Result, ProminenceError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub elevation: i16,
    pub row: usize,
    pub col: usize,
    pub index: usize,
}

impl Cell {
    pub fn new(elevation: i16, row: usize, col: usize, width: usize) -> Self {
        Cell {
            elevation,
            row,
            col,
            index: row * width + col,
        }
    }
}

impl std::cmp::Ord for Cell {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.elevation.cmp(&other.elevation)
    }
}

impl std::cmp::PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ElevationGrid {
    grid: Vec<Vec<i16>>,
    pub width: usize,
    pub height: usize,
}

impl ElevationGrid {
    pub fn new(grid: Vec<Vec<i16>>) -> Result<Self> {
        let height = grid.len();
        if height == 0 {
            return Err(ProminenceError::InvalidDimensions);
        }
        
        let width = grid[0].len();
        if width == 0 {
            return Err(ProminenceError::InvalidDimensions);
        }

        // Validate all rows have same width
        for row in &grid {
            if row.len() != width {
                return Err(ProminenceError::InvalidDimensions);
            }
        }

        Ok(ElevationGrid {
            grid,
            width,
            height,
        })
    }

    pub fn load_from_binary(filename: &str) -> Result<Self> {
        println!("Loading binary file: {}", filename);
        let mut file = File::open(filename)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let total_cells = buffer.len() / 2; // 2 bytes per i16
        
        // Detect dimensions from file size
        let (width, height) = Self::detect_dimensions(total_cells, buffer.len());
        
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
        Self::new(grid)
    }

    fn detect_dimensions(total_cells: usize, buffer_size: usize) -> (usize, usize) {
        // Common DEM dimensions - try professor's layout first
        let common_dims = [
            (4800, 6000), // SRTM 1 arc-second (professor's interpretation)
            (6000, 4800), // SRTM 1 arc-second (our original)
            (1200, 1200), // SRTM 3 arc-second
            (3601, 3601), // SRTM 1 arc-second
            (1201, 1201), // SRTM 3 arc-second
        ];

        for &(w, h) in &common_dims {
            if w * h == total_cells {
                return (w, h);
            }
        }

        // Try square dimensions
        let side = (total_cells as f64).sqrt() as usize;
        if side * side == total_cells {
            return (side, side);
        }

        eprintln!("Warning: Cannot determine grid dimensions from file size");
        eprintln!("File has {} bytes ({} cells), using default 6000x4800", 
                 buffer_size, total_cells);
        (6000, 4800)
    }

    pub fn get_elevation(&self, row: usize, col: usize) -> Option<i16> {
        if row < self.height && col < self.width {
            Some(self.grid[row][col])
        } else {
            None
        }
    }

    pub fn get_neighbor_indices(&self, row: usize, col: usize) -> Vec<usize> {
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

    pub fn is_peak(&self, row: usize, col: usize) -> bool {
        let elevation = match self.get_elevation(row, col) {
            Some(e) => e,
            _ => return false,
        };

        let mut has_lower_neighbor = false;
        
        for dr in -1..=1i32 {
            for dc in -1..=1i32 {
                if dr == 0 && dc == 0 {
                    continue;
                }

                let nr = row as i32 + dr;
                let nc = col as i32 + dc;

                if nr >= 0 && nr < self.height as i32 && nc >= 0 && nc < self.width as i32 {
                    if let Some(neighbor_elev) = self.get_elevation(nr as usize, nc as usize) {
                        if neighbor_elev > elevation {
                            return false; // Has higher neighbor, not a peak
                        }
                        if neighbor_elev < elevation {
                            has_lower_neighbor = true;
                        }
                    }
                }
            }
        }
        
        has_lower_neighbor
    }

    pub fn is_on_boundary(&self, row: usize, col: usize) -> bool {
        row == 0 || row == self.height - 1 || col == 0 || col == self.width - 1
    }

    pub fn get_all_cells(&self, min_elevation: i16) -> Vec<Cell> {
        let mut cells = Vec::new();
        
        for row in 0..self.height {
            for col in 0..self.width {
                let elevation = self.grid[row][col];
                if elevation >= min_elevation {
                    cells.push(Cell::new(elevation, row, col, self.width));
                }
            }
        }
        
        cells.sort();
        cells
    }

    pub fn index_to_coords(&self, index: usize) -> (usize, usize) {
        (index / self.width, index % self.width)
    }
}