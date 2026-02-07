# AutoRAG Data

Build the dataset easily for RAG evaluation & optimization in AutoRAG family.

Desktop application for building and managing RAG evaluation datasets, works perfect with [AutoRAG-Research](https://github.com/NomaDamas/AutoRAG-Research) 
Ingest documents, annotate queries with evidence, and export structured datasets.
Currently supports Visual Document Retrieval type datasets, more in the future.

![sample](./public/usage_example.png)

## Features

- **PDF/image ingestion** with page-level rendering and thumbnail caching
- **Query annotation** — query text, query-to-LLM prompts, generation ground truths
- **Evidence grouping** with graded relevance scoring
- **Export** to CSV, JSON, or ZIP bundles

## Prerequisites

- [Tauri v2 prerequisites](https://tauri.app/start/prerequisites/) (Rust, system deps)
- [pnpm](https://pnpm.io/)
- Docker (for PostgreSQL)

## Quick Start

```bash
# Start the database
make docker-up

# Install dependencies and run
pnpm i
pnpm tauri dev
```

## Tech Stack

Tauri v2 (Rust) · Vue 3 + TypeScript · Vite · Tailwind CSS v4 · Pinia · shadcn-vue · pnpm

## Project Structure

```
src/              Vue frontend
src-tauri/        Rust backend (Tauri commands, DB access)
postgresql/       Docker Compose + init scripts for PostgreSQL + VectorChord
```

## Commands

```bash
pnpm tauri dev        # Dev mode (frontend + backend)
pnpm tauri build      # Production build
pnpm test             # Run Vitest tests
pnpm type-check       # TypeScript checking
pnpm exec eslint .    # Lint frontend
cargo fmt             # Format Rust (in src-tauri/)
cargo clippy          # Lint Rust (in src-tauri/)
```
