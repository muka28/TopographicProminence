# ⛰️ Topographic Prominence Calculator

> Finding the most independent peaks in digital elevation models using efficient union-find algorithms

## What This Does

Ever wondered which mountains are truly "standalone" rather than just high points on a ridge? This calculator finds peaks by their **topographic prominence** - essentially how much you'd have to climb down and back up to reach any higher ground.

Think of it like this: Mount Everest has huge prominence because you'd have to descend thousands of meters before finding a path to anywhere higher. A small bump on Everest's ridge? Not so much.

## Quick Start

```bash
# Compile and run
cargo run --release your_elevation_data.bin

# Or with custom parameters
cargo run --release -- mountain_data.bin
```

## The Problem

Given a grid of elevation data, find the 100 most prominent peaks and their "cols" (the lowest points you'd cross getting to higher terrain).

**Input**: Binary elevation file (16-bit integers)  
**Output**: Peaks sorted by prominence with their key saddle information

```
  prom    row    col   elev   crow   ccol  celev
--------------------------------------------------
  1547   2341   2987   1812   2298   2934    265
  1203   1876   3421   1456     NA     NA     NA
   892   3654   1234   1098   3598   1189    206
```

## How It Works

We use a **union-find (disjoint set)** data structure to efficiently build "drainage basins" - areas that water would flow down from each point.

### The Algorithm

1. **Sort all cells by elevation** (lowest first)
2. **Process each cell** by connecting it to lower neighbors
3. **Track where basins merge** - these become our key saddles
4. **Calculate prominence** as peak height minus saddle height

```rust
// Core idea: build drainage basins from bottom up
for cell in cells_by_ascending_elevation {
    for neighbor in lower_neighbors {
        union_find.connect(cell, neighbor);
    }
}
```

This gives us **O(n log n)** time complexity instead of the naive **O(n³)** approach.

## File Format

Expects binary files with 16-bit little-endian integers:
- Each value represents elevation in meters
- Negative values treated as sea level (0)
- Common formats: SRTM, ASTER GDEM

The code auto-detects common grid dimensions (6000×4800, 1200×1200) or assumes square grids.

## Key Features

- **Efficient**: Processes 28M+ elevation points in under a minute
- **Memory conscious**: Linear space complexity  
- **Robust**: Handles flat summits, boundary conditions, and various grid sizes
- **Accurate**: Implements proper prominence definition from topographic literature

## Understanding the Output

Each peak shows:
- **prom**: Prominence in elevation units
- **row, col**: Peak grid coordinates  
- **elev**: Peak elevation
- **crow, ccol, celev**: Col (saddle) location and elevation
- **NA values**: Peak drains to map boundary (full elevation as prominence)

## Testing

```bash
cargo test
```

Includes tests for:
- Basic prominence calculations
- Boundary detection  
- Peak identification
- Edge cases and small grids

## Algorithm Details

The union-find approach treats each elevation cell as a node. As we process cells from low to high:

1. **Drainage basins form naturally** - water flows to lower ground
2. **Basin merges mark saddles** - where different drainage areas meet  
3. **Peak prominence emerges** - height difference to key saddle

This captures the geographic intuition: prominent peaks have deep saddles separating them from higher terrain.

## Performance Notes

**Typical runtime on modern hardware:**
- 1M cells: ~2 seconds
- 10M cells: ~20 seconds  
- 30M cells: ~60 seconds

**Memory usage scales linearly** with grid size.

## Limitations

- Assumes rectangular grids (no irregular boundaries)
- Binary input format only  
- No coordinate system handling (outputs grid indices)
- Sea level fixed at elevation 0

## Future Ideas

- [ ] Support for other DEM formats (GeoTIFF, ASCII)
- [ ] Geographic coordinate output  
- [ ] Parallel processing for huge datasets
- [ ] Visualization output (SVG/PNG maps)
- [ ] Island prominence calculations

## Contributing

Found a bug? Have ideas? Pull requests welcome!

The main algorithm is in `calculate_prominence_union_find()` - that's where the magic happens.

---

*Built for CSC615 Computational Geometry at [Quinnipiac University]*

**Team Members**: [Tavonga Dutuma and Innocent Chasekwa]
