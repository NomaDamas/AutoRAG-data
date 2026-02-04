# AutoRAG-Data

A Tauri v2 + Vue 3 desktop application for RAG (Retrieval-Augmented Generation) data management.

## Tech Stack

- **Framework**: Tauri v2 (Rust backend)
- **Frontend**: Vue 3 with Composition API
- **Language**: TypeScript
- **Build Tool**: Vite
- **Styling**: Tailwind CSS v4
- **State Management**: Pinia
- **Package Manager**: pnpm

## Key Commands

```bash
# Development
pnpm dev              # Start dev server with Vue DevTools
pnpm tauri dev        # Start Tauri development mode

# Build
pnpm build            # Build frontend
pnpm tauri build      # Build complete Tauri app

# Testing
pnpm test             # Run Vitest tests
pnpm type-check       # TypeScript type checking

# Linting
pnpm exec eslint .    # Lint frontend code

# Rust (in src-tauri/)
cargo fmt             # Format Rust code
cargo check           # Check Rust code
cargo clippy          # Lint Rust code
```

## Project Structure

- `src/` - Vue frontend source
- `src-tauri/` - Rust backend source
- `src-tauri/Cargo.toml` - Rust dependencies
