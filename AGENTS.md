**Agents Guide**

- Purpose: onboarding instructions and conventions for automated agents working in this repository (`excelstream`).
- Location: root of repo. Use this file as the primary source for build/test/lint commands and style rules.

**Build & Test**

- Primary build: `cargo build --all-features` (also available via `make build`). See `Cargo.toml` for optional features.
- Release build: `cargo build --release --all-features` or `make build-release`.
- Run all tests: `cargo test --all-features` or `make test` (Makefile runs `cargo test --all-features`).
- Run a single unit test (by name): `cargo test <test_name> -- --nocapture`.
  - Example (unit test in `src`): `cargo test parse_cell_value -- --nocapture`.
- Run a single integration test (integration tests live in `tests/`): `cargo test --test integration_test <test_name> -- --nocapture`.
  - Example: `cargo test --test integration_test streaming_reads -- --nocapture`.
- Run tests for a specific target or example: `cargo test --lib` or `cargo test --example <name>`.
- Run benchmarks: `cargo bench` or `make bench` (bench harness defined in `Cargo.toml`).

**Format & Lint**

- Formatting: `cargo fmt` (Makefile target `make fmt`). Check-only: `cargo fmt -- --check` or `make fmt-check`.
- Linting: `cargo clippy --all-targets --all-features -- -D warnings` or `make clippy`.
- CI helper: `make ci` runs `fmt-check`, `clippy`, and `test` (mirrors GitHub Actions flow).

**Running examples and tools**

- Build examples: `cargo build --examples --all-features` or `make examples`.
- Run an example: `cargo run --example basic_write` or `make run-example-basic_write`.
- Build docs: `cargo doc --all-features --no-deps` or `make doc`.

**Repository targets & shortcuts**

- Makefile targets: `help`, `fmt`, `fmt-check`, `clippy`, `test`, `build`, `build-release`, `bench`, `examples`, `example-<name>`, `run-example-<name>`, `ci`.
- Primary files: `Cargo.toml`, `Makefile`, `src/lib.rs`, `src/error.rs`, `tests/integration_test.rs`.

**Code Style Guidelines**

- Formatting
  - Use `rustfmt` defaults. Run `cargo fmt` before commits. See `Makefile: fmt` and `fmt-check` targets.
  - Keep line length reasonable (~100 chars) for readability in PR diffs.

- Imports & Module Organization
  - Group imports in this order: 1) std, 2) external crates, 3) internal crate (`crate::...` or `super::...`).
  - Use explicit `use` for commonly referenced items; prefer shorter local names using `as` only when necessary to avoid collisions.
  - Re-export module-level public API from `src/lib.rs` to present a small surface area to consumers.
  - Keep modules small and focused; split large files (over ~400 lines) into submodules.

- Naming Conventions
  - Types, structs, enums, traits: `PascalCase` (e.g., `ExcelWriter`, `CsvWriter`).
  - Functions, variables, module names: `snake_case` (e.g., `write_row`, `parse_csv`).
  - Constants: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_CHUNK_SIZE`).
  - Trait names: use descriptive nouns (no mandatory `Trait` suffix) — prefer `Writer`, `Reader` when appropriate.

- Types & Ownership
  - Prefer references (`&str`, `&[u8]`) in APIs when the callee does not need ownership.
  - Use `String` and owned collections when data must be stored across async boundaries or persisted inside structs.
  - Prefer slices (`&[T]`) and iterator-based APIs over indexing to reduce allocations and copying.
  - Favor small, explicit structs over tuples for clarity in public APIs.

- Error Handling
  - Use the crate's custom error type defined in `src/error.rs` and the `thiserror` crate for conversions.
  - Prefer returning `Result<T, Error>` (crate error) from public APIs.
  - Use the `?` operator to propagate errors; map errors to crate error variants near FFI/IO boundaries.
  - Avoid `unwrap()`/`expect()` in library code. Tests or examples may use `expect` for brevity but prefer `?` or `assert` for test assertions.
  - When converting external errors, preserve context with `.context()`-style messages (or `map_err(|e| Error::Io(format!("...: {}", e)))`).

- Panics & Safety
  - Library code must avoid panics on user input; return errors instead. Panics are acceptable only for internal invariants (use `debug_assert!` where appropriate).
  - For unsafe code (rare), annotate with a short justification and expected invariants. Keep unsafe blocks minimal and well-tested.

- Performance & Memory
  - This project prioritizes low memory and high throughput. Avoid unnecessary allocations and copying.
  - Reuse buffers where feasible, prefer writing directly into buffers rather than building intermediate Strings.
  - Benchmarks and memory-sensitive code live in `benches/` and `examples/`; consult them before making large changes.

- Async & Concurrency
  - Public async APIs use `tokio` features when present; enable `cloud-*` features in `Cargo.toml` as required.
  - Prefer explicit `Send`/`Sync` bounds on types used across threads. Avoid hidden global mutable state.

- Documentation & Comments
  - Document public types and functions with `///` Rustdoc comments. Include short examples where useful.
  - Keep inline comments brief and focused; explain why a non-obvious implementation exists (not what it does).

- Tests
  - Unit tests belong alongside modules in `src/*.rs` using `#[cfg(test)]` and `#[test]`.
  - Integration tests live in `tests/` (see `tests/integration_test.rs`). Use `--test <name>` to run specific integration tests.
  - Use `tempfile` for filesystem artifacts in tests (already in `dev-dependencies`).

- Logging & Diagnostics
  - Use `tracing` or `log` only where necessary; minimize logging in hot code paths. Prefer returning structured errors for callers to log.

**PR & Commit Expectations for Agents**

- Do not create commits unless explicitly instructed by a human. If asked to commit, follow the project's commit message style: short imperative subject and a brief why (1–2 sentences).
- Run `make fmt` and `make clippy` locally before committing; ensure `cargo test --all-features` passes.

**Cursor / Copilot Rules**

- Repository check: no `.cursor/rules/` or `.cursorrules` directories were found.
- No `.github/copilot-instructions.md` file was found. If such files are added later, update this document and follow their directives.

**Helpful file references**

- `Makefile` — convenient targets for format/lint/test (`make fmt`, `make clippy`, `make test`).
- `Cargo.toml` — features and crate metadata (enable `--all-features` in CI/test runs).
- `src/error.rs` — canonical error types and conversions; follow this pattern when adding new errors.
- `tests/integration_test.rs` — reference for integration test layout and names.

If you need to run a quick single-test example:

```bash
# Run a unit test named `parse_csv_row`
cargo test parse_csv_row -- --nocapture

# Run an integration test named `integration_end_to_end` defined in `tests/integration_test.rs`
cargo test --test integration_test integration_end_to_end -- --nocapture
```

If any rule here is ambiguous for a specific change, prefer minimal, safe edits that preserve existing public API and performance characteristics. When in doubt, open an issue or PR describing the proposed change and include benchmarks or memory profiles if the change touches hot code paths.

**WASM / npm demo**

- There is a small in-repo WASM adapter prototype under `wasm_adapter/` that exposes a minimal CSV parsing API via `wasm-bindgen` and a demo in `wasm_adapter/wasm_demo/`.
- Build with `cd wasm_adapter && wasm-pack build --target web --release` (requires `wasm-pack` installed) which produces `wasm_adapter/pkg/` suitable for publishing to npm or serving with a static host.
- For a demo, serve `wasm_adapter/wasm_demo/` (after copying `pkg/` there or adjusting paths) with `python -m http.server` and open the demo in a browser.
- To publish to npm: build with `wasm-pack` then use `npm publish` from `wasm_adapter/pkg/` (follow `wasm-pack` publishing docs).

**Recommended workflow for publishing a web-friendly package**

- 1) Implement minimal adapter crate under `wasm_adapter/` that depends on the workspace `excelstream` crate and exposes JS-friendly functions via `wasm-bindgen`.
- 2) Build with `wasm-pack build --target web --release` to generate the `pkg/` folder.
- 3) Test the demo locally by serving `wasm_adapter/wasm_demo/` and ensuring imports point to `../pkg/` or adjust paths.
- 4) Optionally run `wasm-opt -Oz pkg/yourcrate_bg.wasm -o pkg/yourcrate_bg.wasm` to shrink the binary (requires `binaryen`).
- 5) Publish: either (a) commit demo + pkg to GitHub Pages / Netlify / S3, or (b) `cd pkg && npm publish` to release as an npm package.

**Notes & caveats**

- Keep wasm binary small: avoid pulling large dependencies (ZIP/libs) into the wasm build unless necessary — prefer unzipping XLSX on the JS side using `fflate`/`JSZip` to reduce wasm size.
- Ensure correct HTTP headers when deploying: `.wasm` -> `Content-Type: application/wasm`, `.js` -> `Content-Type: application/javascript`, and set `Content-Encoding` for precompressed assets.
- Use versioned filenames or cache-busting when deploying to CDNs.

---

This file is the canonical guide for automated agents working on this repository. If you want, I can:
1) Scaffold a GitHub Action to build the `wasm_adapter` crate and deploy the demo to GitHub Pages. (Recommended)
2) Create a Makefile target (`make wasm-build`, `make wasm-serve`) and add build/deploy scripts.
3) Publish the `pkg/` to npm (requires npm credentials).

Pick a number and I'll proceed with the required files and CI changes.
