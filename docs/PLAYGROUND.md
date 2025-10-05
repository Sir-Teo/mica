# 🎮 Mica Playground

The Mica Playground is an interactive, browser-based environment for writing, compiling, and exploring Mica code. It provides access to all examples from the repository and allows you to experiment with the language without installing anything locally.

## Features

- **Interactive Code Editor**: Syntax-highlighted editor powered by CodeMirror
- **All Examples Included**: Access all 21+ examples from the repository
- **Multiple Compiler Stages**: View tokens, AST, resolution, IR, and execution output
- **Real-time Updates**: Edit code and see results immediately
- **Modern UI**: Beautiful, responsive design with dark/light theme support

## Accessing the Playground

The playground is available at: **[https://sir-teo.github.io/mica/playground.html](https://sir-teo.github.io/mica/playground.html)**

## How to Use

1. **Select an Example**: Choose from the dropdown menu to load one of the example programs
2. **Edit the Code**: Modify the code in the editor as you like
3. **Choose a Compiler Stage**: Click any of these buttons:
   - **Tokens** - See lexical tokens
   - **AST** - View the abstract syntax tree
   - **Resolve** - Check name resolution and effects
   - **IR** - See the intermediate representation
   - **Run** - Execute the program (coming soon)
4. **View Output**: The right panel shows the compiler output for the selected stage

## Building the Playground

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- Node.js (for generating examples manifest)
- wasm-pack (install via `cargo install wasm-pack`)

### Build Steps

1. **Generate Examples Manifest** (required for loading examples):
   ```bash
   node scripts/generate-examples-manifest.js
   ```

2. **Build WebAssembly Module** (optional, for in-browser compilation):
   ```bash
   bash build-wasm.sh
   ```
   This creates a WASM build in `docs/wasm/` that can be integrated for real-time compilation.

3. **Serve Locally**:
   ```bash
   # Using Python
   cd docs
   python3 -m http.server 8000
   
   # Or using Node.js
   npx serve docs
   ```

4. **Open in Browser**: Navigate to `http://localhost:8000/playground.html`

## WebAssembly Integration (Future)

The playground is designed to support WebAssembly-based in-browser compilation. The WASM module exposes these functions:

- `tokenize(source)` - Tokenize Mica code
- `parse_ast(source, pretty)` - Parse and optionally pretty-print AST
- `resolve_code(source)` - Perform name resolution
- `check_code(source)` - Run exhaustiveness and effect checks
- `lower_code(source)` - Lower to HIR
- `generate_ir(source)` - Generate SSA IR

### Enabling WASM Compilation

To enable real-time compilation in the browser:

1. Build the WASM module (see Build Steps above)
2. The playground will automatically detect and use the WASM module if available
3. Compilation will run entirely in the browser with no server required

## Architecture

```
docs/
├── playground.html          # Main playground page
├── examples-manifest.json   # Auto-generated list of all examples
├── assets/
│   ├── futuristic.css      # Shared styling
│   └── futuristic.js       # Shared JavaScript utilities
└── wasm/                   # WebAssembly module (optional)
    ├── mica.js
    ├── mica_bg.wasm
    └── mica.d.ts

scripts/
├── generate-examples-manifest.js   # Node.js script to scan examples
└── generate-examples-manifest.sh   # Shell script alternative

src/
└── wasm.rs                # WebAssembly bindings for the compiler
```

## Examples Manifest

The `examples-manifest.json` file is automatically generated and contains:

```json
[
  {
    "id": "demo",
    "name": "demo.mica",
    "description": "Basic demo",
    "code": "module demo.core\n...",
    "lines": 12,
    "size": 177
  },
  ...
]
```

This manifest is regenerated whenever examples are added or updated.

## Development

### Adding New Examples

1. Add your `.mica` file to the `examples/` directory
2. Regenerate the manifest:
   ```bash
   node scripts/generate-examples-manifest.js
   ```
3. The new example will automatically appear in the playground dropdown

### Updating the Playground UI

- Edit `docs/playground.html` for HTML structure and JavaScript
- Edit `docs/assets/futuristic.css` for styling (shared across all pages)
- Edit `docs/assets/futuristic.js` for shared utilities

### Testing Locally

Always test locally before deploying:

```bash
cd docs
python3 -m http.server 8000
# Open http://localhost:8000/playground.html
```

## Troubleshooting

### Examples Not Loading

- Ensure `examples-manifest.json` exists in the `docs/` directory
- Check browser console for errors
- Verify the manifest was generated correctly:
  ```bash
  cat docs/examples-manifest.json | head -n 20
  ```

### WASM Module Not Found

- The playground works without WASM (shows instructions instead)
- To enable WASM, run `bash build-wasm.sh`
- Verify `docs/wasm/mica.js` and `docs/wasm/mica_bg.wasm` exist

### CodeMirror Not Loading

- Check internet connection (CodeMirror loads from CDN)
- Or download CodeMirror locally and update the script tags

## Future Enhancements

- [ ] Full WebAssembly-based compilation
- [ ] Syntax highlighting for Mica language
- [ ] Code sharing via URL parameters
- [ ] Save/load custom snippets
- [ ] Step-by-step execution debugger
- [ ] Integration with language server for autocomplete
- [ ] Export to GitHub Gist
- [ ] Interactive tutorials with guided examples

## Contributing

Contributions to the playground are welcome! Some areas where help is needed:

- Custom CodeMirror mode for Mica syntax
- Better error reporting and diagnostics display
- Performance optimizations for large examples
- Mobile-responsive improvements
- Accessibility enhancements

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

The playground is part of the Mica project and follows the same license.
