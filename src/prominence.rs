use crate::{ElevationGrid, Peak, Result, ProminenceError};
use crate::union_find::UnionFind;
use std::time::Instant;

pub struct ProminenceCalculator<'a> {
    grid: &'a ElevationGrid,
}

impl<'a> ProminenceCalculator<'a> {
    pub fn new(grid: &'a ElevationGrid) -> Self {
        ProminenceCalculator { grid }
    }

    pub fn calculate_prominence(
        &self, 
        min_elevation: i16, 
        min_prominence: i16
    ) -> Result<Vec<Peak>> {
        println!("Starting prominence calculation...");
        let start_time = Instant::now();

        let mut uf = UnionFind::new(self.grid.width, self.grid.height);
        let cells = self.grid.get_all_cells(min_elevation);
        let mut processed = vec![false; self.grid.width * self.grid.height];

        self.initialize_union_find(&mut uf, &cells);
        
        println!("Processing {} cells in descending elevation order...", cells.len());
        
        // Process cells from highest to lowest elevation
        self.process_cells(&mut uf, &cells.iter().rev().collect::<Vec<_>>(), &mut processed)?;
        
        println!("Union-find completed in {:.2?}", start_time.elapsed());
        println!("Collecting results...");

        let peaks = uf.collect_peaks(min_prominence);
        println!("Total calculation time: {:.2?}", start_time.elapsed());

        Ok(peaks)
    }

    fn initialize_union_find(&self, uf: &mut UnionFind, cells: &[crate::grid::Cell]) {
        let mut peak_count = 0;
        let mut boundary_count = 0;
        
        for cell in cells {
            // Mark boundary cells
            if self.grid.is_on_boundary(cell.row, cell.col) {
                uf.mark_boundary(cell.index);
                boundary_count += 1;
            }

            // Mark peaks
            if self.grid.is_peak(cell.row, cell.col) {
                uf.mark_as_peak(cell.index, cell.elevation);
                peak_count += 1;
            }
        }
        
println!("Initialized: {} peaks, {} boundary cells", peak_count, boundary_count);
    }

    fn process_cells(
        &self, 
        uf: &mut UnionFind, 
        cells: &[&crate::grid::Cell], 
        processed: &mut [bool]
    ) -> Result<()> {
        for (i, cell) in cells.iter().enumerate() {
            processed[cell.index] = true;
            
            if i % 1_000_000 == 0 && i > 0 {
                self.print_progress(i, cells.len());
            }

            // Connect to processed neighbors at same or lower elevation
            self.connect_to_neighbors(uf, cell, processed)?;
        }
        Ok(())
    }

    fn connect_to_neighbors(
        &self, 
        uf: &mut UnionFind, 
        cell: &crate::grid::Cell, 
        processed: &[bool]
    ) -> Result<()> {
        for neighbor_idx in self.grid.get_neighbor_indices(cell.row, cell.col) {
            if processed[neighbor_idx] {
                let (neighbor_row, neighbor_col) = self.grid.index_to_coords(neighbor_idx);
                
                if let Some(neighbor_elev) = self.grid.get_elevation(neighbor_row, neighbor_col) {
                    // Connect if neighbor is at same or higher elevation (we process from high to low)
                    if neighbor_elev >= cell.elevation {
                        uf.union(cell.index, neighbor_idx, cell.elevation, cell.index);
                    }
                } else {
                    return Err(ProminenceError::ProcessingError(
                        format!("Invalid neighbor coordinates: ({}, {})", neighbor_row, neighbor_col)
                    ));
                }
            }
        }
        Ok(())
    }

    fn print_progress(&self, current: usize, total: usize) {
        let percentage = (current as f64 / total as f64) * 100.0;
        println!("Processed {}/{} cells ({:.1}%)", current, total, percentage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ElevationGrid;

    fn create_test_grid() -> ElevationGrid {
        let grid = vec![
            vec![0, 0, 0, 0, 0],
            vec![0, 2, 1, 2, 0],
            vec![0, 1, 5, 1, 0],
            vec![0, 2, 1, 2, 0],
            vec![0, 0, 0, 0, 0],
        ];
        ElevationGrid::new(grid).unwrap()
    }

    #[test]
    fn test_prominence_calculation() {
        let grid = create_test_grid();
        let calculator = ProminenceCalculator::new(&grid);
        // Use min_elevation=0 to include boundary cells
        let peaks = calculator.calculate_prominence(0, 1).unwrap();
        
        println!("Found {} peaks", peaks.len());
        for peak in &peaks {
            println!("Peak at ({}, {}) elevation={} prominence={}", 
                     peak.row, peak.col, peak.elevation, peak.prominence);
        }
        
        // Should find some peaks
        assert!(!peaks.is_empty(), "Should find at least one peak");
        
        // Central peak should have prominence = 5 (drains to boundary at 0)
        let central_peak = peaks.iter().find(|p| p.elevation == 5);
        if let Some(peak) = central_peak {
            assert_eq!(peak.prominence, 5);
        } else {
            panic!("Should find central peak with elevation 5");
        }
        
        // In this simple test case, only the central peak should be detected
        // Corner peaks with elevation 2 are not local maxima due to the adjacent central peak
        assert_eq!(peaks.len(), 1, "Should find exactly one peak");
        assert_eq!(peaks[0].elevation, 5);
        assert_eq!(peaks[0].prominence, 5);
    }

    #[test]
    fn test_boundary_peaks() {
        let grid = create_test_grid();
        let calculator = ProminenceCalculator::new(&grid);
        let peaks = calculator.calculate_prominence(0, 1).unwrap();
        
        // All peaks in this test case should drain to boundary
        for peak in &peaks {
            if peak.elevation >= 2 {
                // Should have no col information (drains to boundary)
                assert!(peak.col_row.is_none());
            }
        }
    }
}