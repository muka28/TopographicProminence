//! Topographic Prominence Calculator
//! 
//! A clean, modular implementation for calculating topographic prominence
//! from Digital Elevation Model (DEM) data using an improved union-find algorithm.

pub mod peak;
pub mod union_find;
pub mod grid;
pub mod prominence;

pub use peak::Peak;
pub use grid::ElevationGrid;
pub use prominence::ProminenceCalculator;

/// Errors that can occur during prominence calculation
#[derive(Debug)]
pub enum ProminenceError {
    /// I/O error when reading files
    IoError(std::io::Error),
    /// Invalid grid dimensions
    InvalidDimensions,
    /// Invalid elevation data
    InvalidElevation,
    /// Processing error with description
    ProcessingError(String),
}

impl std::fmt::Display for ProminenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProminenceError::IoError(e) => write!(f, "I/O error: {}", e),
            ProminenceError::InvalidDimensions => write!(f, "Invalid grid dimensions"),
            ProminenceError::InvalidElevation => write!(f, "Invalid elevation data"),
            ProminenceError::ProcessingError(msg) => write!(f, "Processing error: {}", msg),
        }
    }
}

impl std::error::Error for ProminenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ProminenceError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ProminenceError {
    fn from(error: std::io::Error) -> Self {
        ProminenceError::IoError(error)
    }
}

/// Result type for prominence calculations
pub type Result<T> = std::result::Result<T, ProminenceError>;