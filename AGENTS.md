# Repository Guidelines

## Project Structure & Module Organization

This is a Rust CLI/server project. Core application code lives in `src/`, with `main.rs` as the binary entry point and `app.rs` wiring the Axum server. Route discovery and parsing logic is under `src/route_builder/`; HTTP, REST, auth, upload, and GraphQL handlers live in `src/handlers/`. The embedded web UI assets are in `src/home/`. Example mock APIs and fixtures are in `mocks/`, documentation is in `docs/`, images are in `images/`, and maintenance scripts are in `scripts/`.

## Build, Test, and Development Commands

- `make build` or `cargo build`: compile the project in debug mode.
- `make build-release` or `cargo build --release`: build an optimized binary.
- `make run` or `cargo run`: run the server locally, using `./mocks` and the default port unless CLI flags override them.
- `make test` or `cargo test`: run all unit tests.
- `make clippy`: run Clippy with warnings treated as errors.
- `make fmt` / `make fmt-check`: format code or verify formatting.
- `make check-all`: run tests, Clippy, and formatting checks.

## Coding Style & Naming Conventions

Use Rust 2024 idioms and `rustfmt` defaults; do not hand-format around `cargo fmt`. Keep modules focused by feature area, matching the existing `route_*` and `*_handlers` naming patterns. Prefer explicit, descriptive function and test names in `snake_case`. Avoid adding dependencies unless they materially simplify the implementation and fit the project scope.

## Testing Guidelines

Tests are primarily inline unit tests inside `#[cfg(test)] mod tests` blocks near the code they verify. Add regression tests for route parsing, config behavior, handler edge cases, and mock-file conventions before changing behavior. Use `tempfile` for filesystem-dependent tests instead of relying on global local state. Run `make check-all` before submitting changes.

## Commit & Pull Request Guidelines

Recent history uses Conventional Commit prefixes such as `fix:`, `feat:`, `docs:`, and `chore:`. Keep commit subjects short and behavior-focused, for example `fix: preserve hot reload on file updates`. Pull requests should describe the user-visible change, list verification performed, link related issues when applicable, and include screenshots only for web UI changes.

## Security & Configuration Tips

Do not commit secrets, tokens, or private mock data. Treat files in `mocks/` as examples only. When touching auth or upload behavior, verify both successful and rejected paths, and document any new configuration in `docs/` and `README.md`.
