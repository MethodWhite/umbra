# Contributing to Umbra

Thank you for your interest in contributing to Umbra! This document provides guidelines and instructions for contributing.

## Development Environment

### Prerequisites
- Rust toolchain (stable): `rustup install stable`
- System dependencies (Ubuntu/Debian):
  ```bash
  sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libasound2-dev libsqlcipher-dev
  ```
- For other platforms, see [Tauri prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites).

### Setup
1. Clone the repository:
   ```bash
   git clone https://github.com/anomalyco/umbra.git
   cd umbra
   ```
2. Build the project:
   ```bash
   cargo build --release --bin umbra-gui
   ```
3. Run tests:
   ```bash
   cargo test --lib
   ```

## Code Style

- **Rustfmt**: All Rust code must be formatted with `rustfmt`. Run `cargo fmt` before committing.
- **Clippy**: Run `cargo clippy --all-targets --all-features -- -D warnings` to catch common mistakes and enforce best practices.
- Follow the existing code conventions and patterns used throughout the project.

## Pull Request Process

1. Ensure your fork is up to date with the upstream `master` branch.
2. Create a feature branch: `git checkout -b feat/my-feature`.
3. Make your changes, adhering to the code style guidelines.
4. Write or update tests as needed.
5. Run `cargo test --lib` and ensure all tests pass.
6. Run `cargo fmt` and `cargo clippy --all-targets --all-features -- -D warnings`.
7. Update `CHANGELOG.md` with your changes under the `[Unreleased]` section.
8. Commit with a clear, descriptive message and push your branch.
9. Open a pull request against the `master` branch.

## Zone-Based Development

Umbra uses zone-based development to organize the codebase into clearly separated responsibility areas. Before contributing, please read [`DEVELOPMENT.md`](./DEVELOPMENT.md) to understand the zone architecture and ensure your changes respect module boundaries.

## Security Reporting

If you discover a security vulnerability, **do not** open a public issue. Instead, refer to [`SECURITY.md`](./SECURITY.md) for our security policy and reporting instructions. We take all security reports seriously and will respond promptly.
