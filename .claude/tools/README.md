# Codebase Mapper

Standalone tool to map codebase dependencies. Extracts function definitions and call relationships from Rust code and outputs JSON for LLM consumption.

## Features

- **Dual-mode parsing**: Uses tree-sitter for accurate AST parsing when available, with regex fallback
- **Function extraction**: Identifies functions, methods, and trait methods
- **Call relationship mapping**: Tracks direct calls, method calls, and macro invocations
- **Module structure**: Builds module hierarchy from file paths
- **Reverse call graph**: Enables querying "what calls X?"
- **JSON output**: Structured format for LLM consumption

## Installation

```bash
pip install -r requirements.txt
```

The tool will attempt to use tree-sitter if available. If tree-sitter grammar compilation fails, it falls back to regex-based parsing.

**For tree-sitter mode**, you need:
- Git (to clone the grammar repository)
- A C compiler (to compile the grammar on Windows: Visual Studio Build Tools or MinGW)

**For regex mode** (fallback), no additional dependencies needed.

## Usage

Map a Rust codebase:

```bash
python codebase_mapper.py <repo_path> [output_file] [--compact]
```

Examples:

```bash
# Map with pretty-printed JSON (larger, human-readable)
python codebase_mapper.py /workspace/ELAI\.elai_cc elai_dependency_graph.json

# Map with compact JSON (smaller, single line)
python codebase_mapper.py /workspace/ELAI\.elai_cc elai_dependency_graph.json --compact
```

## Current Results (elai codebase)

- **Functions**: 4,990
- **Call relationships**: 84,535
- **Modules**: 170
- **File size**: 17.8 MB (compact mode) / 24.5 MB (pretty-printed)

**Note**: Use `--compact` flag for smaller file size (single-line JSON). Pretty-printed JSON is ~40% larger but easier to read.

## Limitations

**Regex mode (fallback)**:
- May produce false positives (e.g., keywords detected as function calls)
- Less accurate than tree-sitter AST parsing
- Doesn't handle complex Rust syntax perfectly

**Tree-sitter mode**:
- Requires C compiler for grammar compilation
- Initial setup takes longer (cloning + compiling grammar)

## Output Format

The tool generates a JSON file with:

- **functions**: Map of `"file:function_name"` -> function info (name, file, line, signature, kind)
- **calls**: List of call relationships (caller, callee, file, line, call_type)
- **modules**: Module structure with function lists
- **reverse_calls**: Reverse mapping (callee -> list of callers)
- **metadata**: Summary statistics

## Example Query

Once you have the dependency graph, you can query it:

```python
import json

with open('elai_dependency_graph.json') as f:
    graph = json.load(f)

# Find what calls a specific function
function_name = "parse"
callers = graph['reverse_calls'].get(function_name, [])
print(f"'{function_name}' is called by: {callers}")

# Find all functions in a module
for module_path, module_info in graph['modules'].items():
    if 'knowledge' in module_path:
        print(f"{module_path}: {module_info['functions']}")
```

## Integration with LLM

Provide the JSON output to an LLM with context like:

> "Based on this dependency graph, if I trigger the user login flow, does it eventually reach the legacy database module?"

The LLM can traverse the graph to answer reachability questions.

## Query Tool

Use `query_graph.py` to explore the generated dependency graph:

```bash
# Print statistics
python query_graph.py elai_dependency_graph.json stats

# Find what calls a function
python query_graph.py elai_dependency_graph.json callers parse

# Find what a function calls
python query_graph.py elai_dependency_graph.json callees main

# Find where a function is defined
python query_graph.py elai_dependency_graph.json locate query

# Find functions in modules matching a pattern
python query_graph.py elai_dependency_graph.json module knowledge
```
