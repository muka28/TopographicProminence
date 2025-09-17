#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peak {
    pub row: usize,
    pub col: usize,
    pub elevation: i16,
    pub prominence: i16,
    pub col_row: Option<usize>,
    pub col_col: Option<usize>,
    pub col_elevation: Option<i16>,
}

impl Peak {
    pub fn new(row: usize, col: usize, elevation: i16) -> Self {
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

    pub fn with_col(mut self, row: usize, col: usize, elevation: i16) -> Self {
        self.col_row = Some(row);
        self.col_col = Some(col);
        self.col_elevation = Some(elevation);
        self
    }

    pub fn with_prominence(mut self, prominence: i16) -> Self {
        self.prominence = prominence;
        self
    }
}

impl std::fmt::Display for Peak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (crow_str, ccol_str, celev_str) = match (self.col_row, self.col_col, self.col_elevation) {
            (Some(crow), Some(ccol), Some(celev)) => {
                (format!("{:6}", crow), format!("{:6}", ccol), format!("{:6}", celev))
            }
            _ => ("    NA".to_string(), "    NA".to_string(), "    NA".to_string())
        };

        write!(f, "{:6} {:6} {:6} {:6} {} {} {}",
               self.prominence, self.row, self.col, self.elevation,
               crow_str, ccol_str, celev_str)
    }
}