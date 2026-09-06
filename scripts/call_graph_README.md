# Call Graph / Dependency Graph Generator

A Python tool for generating call graphs and dependency graphs from Python code.

## Features

- **Call Graphs**: Visualize function-to-function call relationships
- **Import Graphs**: Show import dependencies between modules
- **Module Graphs**: High-level module dependency view
- **Multiple Output Formats**: DOT (Graphviz) and JSON

## Usage

```bash
python call_graph.py <path> [--output OUTPUT] [--format FORMAT] [--type TYPE]
```

### Arguments

- `path`: Path to Python file or directory to analyze (required)
- `--output`, `-o`: Output file path (default: graph.dot)
- `--format`, `-f`: Output format - `dot` or `json` (default: dot)
- `--type`, `-t`: Graph type - `call`, `import`, or `module` (default: call)

### Examples

Generate a call graph for a directory:
```bash
python call_graph.py my_project/ --output call_graph.dot
```

Generate an import dependency graph in JSON:
```bash
python call_graph.py my_module.py --format json --type import --output imports.json
```

Generate a module-level dependency graph:
```bash
python call_graph.py my_project/ --type module --output module_deps.dot
```

## Visualization

To visualize DOT files, you need Graphviz installed:

```bash
# Install Graphviz (Windows)
choco install graphviz

# Or download from: https://graphviz.org/download/

# Convert DOT to PNG
dot -Tpng graph.dot -o graph.png
```

## Output Types

### Call Graph (`--type call`)
Shows function-to-function call relationships within the analyzed code.

### Import Graph (`--type import`)
Shows what modules/packages each file imports from.

### Module Graph (`--type module`)
Shows dependencies between modules within the project (only includes internal modules).

## Requirements

- Python 3.8+
- No external dependencies (uses only standard library)
- Graphviz (for visualization, optional)

## Notes

- Skips test files and `__pycache__` directories
- Handles both sync and async functions
- Preserves class hierarchy in function names (e.g., `ClassName.method_name`)
