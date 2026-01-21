/// Format a markdown table with aligned columns.
///
/// Takes raw table text and returns formatted table with proper padding.
pub fn format_table(input: &str) -> String {
    let lines: Vec<&str> = input.lines().collect();
    if lines.is_empty() {
        return input.to_string();
    }

    // Parse all rows into cells
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut separator_row_index: Option<usize> = None;

    for (idx, line) in lines.iter().enumerate() {
        let cells = parse_row(line);
        if is_separator_row(&cells) {
            separator_row_index = Some(idx);
        }
        rows.push(cells);
    }

    if rows.is_empty() {
        return input.to_string();
    }

    // Calculate max width for each column
    let num_columns = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut col_widths: Vec<usize> = vec![0; num_columns];

    for (idx, row) in rows.iter().enumerate() {
        // Skip separator row when calculating widths
        if separator_row_index == Some(idx) {
            continue;
        }
        for (col, cell) in row.iter().enumerate() {
            col_widths[col] = col_widths[col].max(cell.len());
        }
    }

    // Ensure minimum width of 3 for separator dashes
    for width in &mut col_widths {
        if *width < 3 {
            *width = 3;
        }
    }

    // Rebuild the table
    let mut output = String::new();
    for (idx, row) in rows.iter().enumerate() {
        if separator_row_index == Some(idx) {
            output.push_str(&build_separator_row(&col_widths));
        } else {
            output.push_str(&build_data_row(row, &col_widths));
        }
        output.push('\n');
    }

    // Remove trailing newline to match input format
    if !input.ends_with('\n') && output.ends_with('\n') {
        output.pop();
    }

    output
}

/// Parse a single row into cells, handling the | delimiters.
fn parse_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();

    // Remove leading and trailing pipes if present
    let inner = trimmed
        .strip_prefix('|')
        .unwrap_or(trimmed)
        .strip_suffix('|')
        .unwrap_or(trimmed);

    inner
        .split('|')
        .map(|cell| cell.trim().to_string())
        .collect()
}

/// Check if a row is a separator row (contains only dashes and colons).
fn is_separator_row(cells: &[String]) -> bool {
    if cells.is_empty() {
        return false;
    }
    cells.iter().all(|cell| {
        !cell.is_empty() && cell.chars().all(|c| c == '-' || c == ':')
    })
}

/// Build a data row with proper padding.
fn build_data_row(cells: &[String], col_widths: &[usize]) -> String {
    let mut parts: Vec<String> = Vec::new();

    for (i, width) in col_widths.iter().enumerate() {
        let cell = cells.get(i).map(|s| s.as_str()).unwrap_or("");
        parts.push(format!(" {:<width$} ", cell, width = width));
    }

    format!("|{}|", parts.join("|"))
}

/// Build a separator row with dashes.
fn build_separator_row(col_widths: &[usize]) -> String {
    let parts: Vec<String> = col_widths
        .iter()
        .map(|&width| format!("-{}-", "-".repeat(width)))
        .collect();

    format!("|{}|", parts.join("|"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_simple_table() {
        let input = "| Name | Age | City |
|---|---|---|
| Alice | 30 | New York |";

        let expected = "| Name  | Age | City     |
|-------|-----|----------|
| Alice | 30  | New York |";

        assert_eq!(format_table(input), expected);
    }

    #[test]
    fn test_format_uneven_columns() {
        let input = "| A | B |
|-|-|
| Short | Much longer text |";

        let expected = "| A     | B                |
|-------|------------------|
| Short | Much longer text |";

        assert_eq!(format_table(input), expected);
    }

    #[test]
    fn test_empty_input() {
        assert_eq!(format_table(""), "");
    }

    #[test]
    fn test_parse_row() {
        let cells = parse_row("| Name | Age |");
        assert_eq!(cells, vec!["Name", "Age"]);
    }

    #[test]
    fn test_is_separator_row() {
        assert!(is_separator_row(&vec!["---".to_string(), "---".to_string()]));
        assert!(is_separator_row(&vec![":--".to_string(), "--:".to_string()]));
        assert!(!is_separator_row(&vec!["Name".to_string(), "Age".to_string()]));
    }
}
