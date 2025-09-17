use crate::Peak;
use std::collections::HashMap;

pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    peak_elevation: Vec<i16>,
    peak_index: Vec<Option<usize>>,
    key_saddle_elevation: Vec<i16>,
    saddle_index: Vec<Option<usize>>,
    drains_to_boundary: Vec<bool>,
    width: usize,
    height: usize,
}

impl UnionFind {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        UnionFind {
            parent: (0..size).collect(),
            rank: vec![0; size],
            peak_elevation: vec![i16::MIN; size],
            peak_index: vec![None; size],
            key_saddle_elevation: vec![i16::MIN; size],
            saddle_index: vec![None; size],
            drains_to_boundary: vec![false; size],
            width,
            height,
        }
    }

    pub fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    pub fn union(&mut self, x: usize, y: usize, merge_elevation: i16, merge_index: usize) {
        let root_x = self.find(x);
        let root_y = self.find(y);

        if root_x == root_y {
            return;
        }

        self.merge_components(root_x, root_y, merge_elevation, merge_index);
    }

    fn merge_components(&mut self, root_x: usize, root_y: usize, merge_elevation: i16, merge_index: usize) {
        // Always merge smaller rank into larger rank
        let (parent_root, child_root) = if self.rank[root_x] >= self.rank[root_y] {
            (root_x, root_y)
        } else {
            (root_y, root_x)
        };

        // Update rank
        if self.rank[root_x] == self.rank[root_y] {
            self.rank[parent_root] += 1;
        }

        // Merge drainage status
        self.drains_to_boundary[parent_root] = 
            self.drains_to_boundary[parent_root] || self.drains_to_boundary[child_root];

        // Keep the higher peak - this is crucial!
        if self.peak_elevation[child_root] > self.peak_elevation[parent_root] {
            self.peak_elevation[parent_root] = self.peak_elevation[child_root];
            self.peak_index[parent_root] = self.peak_index[child_root];
        } else if self.peak_index[parent_root].is_none() && self.peak_index[child_root].is_some() {
            // If parent has no peak but child does, use child's peak
            self.peak_elevation[parent_root] = self.peak_elevation[child_root];
            self.peak_index[parent_root] = self.peak_index[child_root];
        }

        // Update key saddle - when processing high to low, the merge elevation is the saddle
        // Only update if this creates a lower escape route than current best
        if !self.drains_to_boundary[parent_root] {
            if self.key_saddle_elevation[parent_root] == i16::MIN || 
               merge_elevation < self.key_saddle_elevation[parent_root] {
                self.key_saddle_elevation[parent_root] = merge_elevation;
                self.saddle_index[parent_root] = Some(merge_index);
            }
        }

        // Consider child's key saddle - use the lower (better escape route)
        if !self.drains_to_boundary[parent_root] && 
           self.key_saddle_elevation[child_root] != i16::MIN &&
           (self.key_saddle_elevation[parent_root] == i16::MIN || 
            self.key_saddle_elevation[child_root] < self.key_saddle_elevation[parent_root]) {
            self.key_saddle_elevation[parent_root] = self.key_saddle_elevation[child_root];
            self.saddle_index[parent_root] = self.saddle_index[child_root];
        }

        // Perform the union
        self.parent[child_root] = parent_root;
    }

    pub fn mark_as_peak(&mut self, index: usize, elevation: i16) {
        let root = self.find(index);
        if elevation > self.peak_elevation[root] {
            self.peak_elevation[root] = elevation;
            self.peak_index[root] = Some(index);
        }
    }

    pub fn mark_boundary(&mut self, index: usize) {
        let root = self.find(index);
        self.drains_to_boundary[root] = true;
    }

    fn index_to_coords(&self, index: usize) -> (usize, usize) {
        (index / self.width, index % self.width)
    }

    pub fn collect_peaks(&mut self, min_prominence: i16) -> Vec<Peak> {
        let mut peak_map = HashMap::new();
        let mut stats = ComponentStats::new();

        // Process each grid cell to find unique components
        for i in 0..self.parent.len() {
            let root = self.find(i);
            
            if let Some(peak_idx) = self.peak_index[root] {
                // Only process each component once
                if peak_map.contains_key(&peak_idx) {
                    continue;
                }
                
                stats.total_components += 1;
                
                let peak_elev = self.peak_elevation[root];
                
                // Calculate prominence correctly
                let prominence = self.calculate_prominence(root, peak_elev);
                
                if prominence >= min_prominence && prominence > 0 {
                    stats.valid_peaks += 1;
                    let peak = self.create_peak(peak_idx, peak_elev, prominence, root);
                    peak_map.insert(peak_idx, peak);
                }
            }
        }

        stats.print(min_prominence);

        let mut peaks: Vec<Peak> = peak_map.into_values().collect();
        peaks.sort_by(|a, b| b.prominence.cmp(&a.prominence));
        peaks
    }

    fn calculate_prominence(&self, root: usize, peak_elevation: i16) -> i16 {
        if self.drains_to_boundary[root] {
            // Peak drains to boundary (effectively sea level = 0)
            peak_elevation
        } else {
            // Peak is enclosed - prominence is height above key saddle
            let saddle_elevation = self.key_saddle_elevation[root];
            if saddle_elevation > i16::MIN {
                peak_elevation - saddle_elevation
            } else {
                0 // This shouldn't happen for properly connected components
            }
        }
    }

    fn create_peak(&self, peak_idx: usize, peak_elev: i16, prominence: i16, root: usize) -> Peak {
        let (peak_row, peak_col) = self.index_to_coords(peak_idx);
        
        let mut peak = Peak::new(peak_row, peak_col, peak_elev).with_prominence(prominence);
        
        // Add col (saddle) information for enclosed peaks
        if !self.drains_to_boundary[root] {
            if let Some(saddle_idx) = self.saddle_index[root] {
                let (saddle_row, saddle_col) = self.index_to_coords(saddle_idx);
                let saddle_elev = self.key_saddle_elevation[root];
                peak = peak.with_col(saddle_row, saddle_col, saddle_elev);
            }
        }
        
        peak
    }
}

struct ComponentStats {
    total_components: usize,
    valid_peaks: usize,
}

impl ComponentStats {
    fn new() -> Self {
        ComponentStats {
            total_components: 0,
            valid_peaks: 0,
        }
    }

    fn print(&self, min_prominence: i16) {
        println!("Found {} components with peaks, {} valid peaks with prominence >= {}", 
                 self.total_components, self.valid_peaks, min_prominence);
    }
}