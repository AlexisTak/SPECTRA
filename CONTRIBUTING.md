# Contributing to Cekarna

Thank you for your interest in contributing to Cekarna! This document provides guidelines for contributing to the project.

## Code of Conduct

- Be respectful and inclusive
- Accept constructive feedback gracefully
- Focus on what is best for the community
- Show empathy for other community members

## Development Setup

### Prerequisites

- **Node.js** ≥ 20
- **Rust** (stable channel via rustup)
- **Tauri CLI** ≥ 2.0
- **Ollama** (optional, for AI features)

### Installation

```bash
# Clone the repository
git clone https://github.com/cekarna/enquetes.git
cd enquetes

# Install dependencies
npm install
```

### Running in Development

```bash
# Run in dev mode (Tauri + Next.js)
npm run dev

# Or run separately:
npm run tauri dev    # Terminal 1: Tauri backend
npm run dev:next     # Terminal 2: Next.js frontend
```

## Project Structure

```
cekarna-enquetes/
├── app/                      # Next.js 16 app router
├── lib/                     # TypeScript utilities
├── components/              # React components
├── types/                   # Type definitions
├── src-tauri/               # Rust backend
│   ├── src/
│   │   ├── lib.rs           # Main entry point
│   │   ├── database.rs      # SQLite schema (24 tables)
│   │   └── commands/        # Tauri commands
│   └── Cargo.toml
└── out/                     # Next.js export output
```

## Development Workflow

### 1. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/your-bug-fix
```

### 2. Make Changes

- Follow the existing code style
- Write clear, descriptive commit messages
- Add tests for new features
- Update documentation as needed

### 3. Run Tests

```bash
# Run TypeScript tests
npm run test

# Run Rust tests
cd src-tauri
cargo test
```

### 4. Build

```bash
# Build for current platform
npm run build

# Build Tauri app
npm run tauri build
```

### 5. Submit a Pull Request

- Fill out the PR template
- Reference any related issues
- Request review from maintainers

## Code Style

### Rust

- Follow [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/)
- Use `rustfmt` for formatting
- Use `clippy` for linting

```bash
cargo fmt
cargo clippy --all-targets --all-features
```

### TypeScript/React

- Follow [Next.js Code Style](https://nextjs.org/docs/app/building-your-application/routing/column-layouts#code-style)
- Use TypeScript strict mode
- Follow React 19 patterns

## Type Safety

All Tauri commands are documented in `lib/tauri-bridge.ts`. When adding new commands:

1. Add the Rust command in `src-tauri/src/commands/`
2. Add the TypeScript wrapper in `lib/tauri-bridge.ts`
3. Update type definitions in `types/` if needed

## Database Changes

When modifying the database schema:

1. Update `src-tauri/src/database.rs`
2. Add migration scripts if needed
3. Update documentation in `PROJET.md`

## Documentation

- Update `PROJET.md` for major changes
- Update `QUICKSTART.md` for setup instructions
- Update this file for contribution guidelines

## Pull Request Checklist

- [ ] Tests pass (`npm run test`, `cargo test`)
- [ ] Code is formatted (`cargo fmt`, `npm run format`)
- [ ] Lints pass (`cargo clippy`, `npm run lint`)
- [ ] Documentation updated
- [ ] No new warnings or errors

## Issue Reporting

When reporting issues:

1. Use the provided issue template
2. Include steps to reproduce
3. Include expected vs actual behavior
4. Include environment details (OS, Rust version, Node version)

## Contact

- GitHub Issues: [https://github.com/cekarna/enquetes/issues](https://github.com/cekarna/enquetes/issues)
- Project Docs: See `PROJET.md`

Thank you for contributing!
