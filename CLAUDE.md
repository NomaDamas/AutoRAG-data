# AutoRAG-Data

Tauri v2 + Vue 3/TypeScript desktop app for RAG dataset management. Tailwind v4, Pinia, shadcn-vue, pnpm.

## Commands

```bash
pnpm tauri dev          # Dev mode
pnpm tauri build        # Production build
pnpm test               # Vitest
pnpm type-check         # TS checking
pnpm exec eslint .      # Lint frontend
cargo fmt               # Format Rust (in src-tauri/)
cargo clippy            # Lint Rust (in src-tauri/)
```

## Structure

- `src/` — Vue frontend: components in `src/components/ui/` (shadcn-vue/reka-ui), stores in `src/stores/` (Pinia)
- `src-tauri/` — Rust backend: Tauri commands in `src-tauri/src/commands/`, DB models/queries alongside
- `postgresql/` — Docker Compose + init scripts for PostgreSQL + VectorChord
