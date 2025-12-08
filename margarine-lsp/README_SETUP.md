# Margarine LSP - Language Server Protocol Implementation

This project contains the Margarine Language Server Protocol (LSP) implementation and VS Code extension.

## Project Structure

```
margarine-lsp/
├── lsp/                      # Rust LSP Server
│   ├── Cargo.toml           # Rust dependencies
│   ├── Cargo.lock
│   ├── rust-toolchain.toml
│   └── src/                 # LSP server source code
│
└── vscode-client/           # VS Code Extension
    ├── package.json         # Extension metadata
    ├── tsconfig.json        # TypeScript configuration
    ├── esbuild.js           # Build configuration
    ├── src/                 # Extension source code
    │   └── src/
    │       └── extension.ts # Main extension file
    └── node_modules/        # Dependencies
```

## Building the LSP Server

### Prerequisites
- Rust nightly toolchain
- The Margarine compiler dependencies in the parent directory

### Build Steps

```bash
# From the lsp/ directory
cd lsp
cargo build --release
```

The binary will be at `lsp/target/release/margarine-lsp`

## Building the VS Code Extension

### Prerequisites
- Node.js 18+
- pnpm package manager

### Build Steps

```bash
# From the vscode-client/ directory
cd vscode-client

# Install dependencies
pnpm install

# Build the extension
pnpm compile

# Watch mode during development
pnpm watch
```

The extension will be packaged in `vscode-client/dist/extension.js`

## Development Setup

1. **Build the LSP server first:**
   ```bash
   cd lsp
   cargo build
   ```

2. **Install extension dependencies:**
   ```bash
   cd vscode-client
   pnpm install
   ```

3. **Watch for changes:**
   ```bash
   # In vscode-client/
   pnpm watch
   ```

4. **Open the extension in VS Code:**
   - Open the project in VS Code
   - Press `F5` to launch in debug mode
   - A new VS Code window will open with the extension loaded
   - Open a `.mar` file to test the LSP

## Configuration

The LSP server looks for these environment variables:

- `SERVER_PATH`: Path to the LSP binary (defaults to `margarine-lsp`)
- `RUST_LOG`: Log level for the server (e.g., `debug`, `info`)

## Packaging the Extension

To create a `.vsix` file for distribution:

```bash
cd vscode-client
pnpm package
```

This will create a `.vsix` file that can be distributed and installed.

## LSP Features

The current implementation supports:

- Text document synchronization
- Diagnostics reporting
- Semantic tokens
- Completion
- Hover information
- Go to definition
- Document symbols
- References
- Rename

Add more features by extending the `Backend` implementation in `lsp/src/main.rs`.

## Integration with Margarine Compiler

The LSP server uses the following Margarine modules:

- `lexer` - Tokenization
- `parser` - AST parsing
- `semantic_analysis` - Type checking and symbol resolution
- `common` - Common utilities
- `errors` - Error formatting

These are imported via relative paths in `lsp/Cargo.toml`. Ensure the Margarine compiler is built and available in the parent directory.

## Troubleshooting

### "command not found: margarine-lsp"
Make sure the LSP binary is in your PATH or set `SERVER_PATH` environment variable.

### Extension not loading
Check the "Output" panel in VS Code for errors. Look for the "Margarine Language Server trace" channel.

### Build failures
Ensure Rust nightly is installed: `rustup toolchain install nightly`
