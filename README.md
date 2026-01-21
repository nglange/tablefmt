# tablefmt - Markdown Table Formatter for Zed

A Zed extension that formats markdown tables with properly aligned columns.

## Installation

Install from the Zed extension marketplace by searching for "tablefmt" or "Markdown Table Formatter".

## Usage

1. Select a markdown table in your document
2. Run **Editor: Format Selection** (or use the keyboard shortcut)
3. The table will be formatted with aligned columns

## Example

**Before:**
```markdown
| Name | Age | City |
|---|---|---|
| Alice | 30 | New York |
| Bob | 25 | Los Angeles |
```

**After:**
```markdown
| Name  | Age | City        |
|-------|-----|-------------|
| Alice | 30  | New York    |
| Bob   | 25  | Los Angeles |
```

## Features

- Aligns all columns based on the widest cell in each column
- Properly formats separator rows with appropriate dash widths
- Works with any valid markdown table format

## Building from Source

### Extension (WASM)

```bash
cargo build --release --target wasm32-wasip1
```

### LSP Binary

```bash
cd lsp
cargo build --release
```

## License

MIT License - see [LICENSE](LICENSE) for details.
