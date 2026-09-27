# Repository Guidelines

## Project Overview

Rust reconstruction of the 1996 Mattmoss Windows screensaver, delivered as a standalone desktop animation for macOS, Windows and Linux. This is reconstructed code, not recovered original source or OS screensaver integration. Preserve the distinction between historical arithmetic fidelity and intentionally modern presentation.

## Architecture & Data Flow

- `mattmoss-core` is dependency-free, synchronous CPU rendering. `field` implements the recovered integer kernel; `Animation` owns seeded RNG, scatter traversal, palette/timer state and reusable pixel buffers. `SmoothAnimation` wraps it with palette interpolation and scene crossfades.
- Desktop imports **`SmoothAnimation as Animation`**. Inputs/elapsed time → animation state → borrowed 640×480 RGBA8 buffer → reusable Macroquad image/texture → letterboxed window. Keep clock, keyboard, window and GPU concerns in desktop.
- The headless example consumes the same smooth renderer and writes RGB P6 PPM without a display. Default full-image mode prepares complete fields; interlaced mode retains progressive historical drawing.
- State is owned by animation structs and desktop-local variables. Inject a seed through `new(seed)` and simulation time through `advance(seconds)`; no service container or global application state. Async is limited to Macroquad's `next_frame().await`, not an async core or Tokio runtime.

## Key Directories

- `crates/core/src/`: numerical kernel, both animation implementations and inline unit tests.
- `crates/core/tests/`: embedded original-x86 golden vectors; inspect relevant cases rather than loading the entire large fixture file.
- `crates/core/examples/`: headless export utility, not a separate scripting framework.
- `crates/desktop/src/`: native CLI, controls and rendering loop.
- `.github/workflows/`: native build/test matrix and executable artifact uploads.

## Development Commands

Run from the repository root:

```sh
cargo build --release --locked
cargo run --release --locked -- --seed 1996
cargo test --workspace --locked
cargo test -p mattmoss-core --test original_x86 --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked
cargo run -p mattmoss-core --example render --release --locked -- preview.ppm
cargo run --release --locked -- --seed 1996 --smoke-test
```

The workspace defaults to **desktop only**: use `--workspace` for the complete test suite. Clippy is a recommended local check, not an existing CI gate. The desktop smoke flag exits after 180 frames and still requires a working graphical display; use the core example for headless execution. The exporter writes PPM regardless of the output extension.

## Code Conventions & Common Patterns

- Follow rustfmt and existing Rust naming; retain domain spellings such as `colour`, `COLOURS` and `interlaced`.
- Preserve explicit wrapping arithmetic, truncation, Microsoft C RNG behavior and the deliberate palette constant `3.14` with its narrow Clippy allowance. At zero squared distance, `field` returns the previous field value: traversal order is observable, so independent pixel parallelization is not automatically safe.
- Reuse frame buffers and return borrowed pixel slices. `pixels()` refreshes output storage but must not advance simulation time. Pause works by skipping `advance`, not by stopping rendering.
- `advance` ignores nonfinite/nonpositive time and caps each call at 0.25 seconds. Determinism requires the same seed, controls and time steps, not merely equal total elapsed time.
- Preserve retained indices across ordinary pattern changes; toggling interlace clears indices and starts a new pattern. Smooth transitions retain the displayed frame and fade over one simulation second, approximately four wall-clock seconds at default 0.25× speed.
- The exporter propagates I/O errors with `std::io::Result` and `?`. Desktop uses manual CLI parsing, prints argument errors and returns from `main`; those branches do not explicitly set a nonzero exit status. Match the local error model rather than adding an unrelated framework.

## Important Files

- `crates/core/src/lib.rs`: arithmetic and animation contracts; `crates/desktop/src/main.rs`: window entry point, CLI and controls.
- `Cargo.toml`: workspace/default member and release profile; crate manifests: dependencies; `Cargo.lock`: reproducible dependency resolution.
- `.github/workflows/build.yml`: stable-Rust tests and release builds on Linux, Windows and macOS; no GUI, formatting, lint or coverage gate.
- Read `REVERSE_ENGINEERING.md` before changing field arithmetic, RNG, palette or historical timing: it records reconstruction evidence and intentional adaptations.
- Read `README.md` for user-facing commands/controls and `VALIDATION.md` for historical verification limits. Historical pass claims are not fresh results. `preview.png` is a reference asset, not an automated golden-image fixture.

## Runtime/Tooling Preferences

Use stable Rust and Cargo (edition 2021); no numeric MSRV or pinned toolchain is declared. Keep `--locked` for normal builds/tests and retain the committed lockfile. Desktop pins Macroquad exactly to `0.4.14` with default features disabled; preserve that policy unless intentionally updating dependencies.

Native builds need a platform linker: Xcode command-line tools on macOS, MSVC/Visual Studio C++ tools on Windows, or a C linker on Linux. GUI execution also needs a desktop graphics environment (OpenGL support on Linux). Python/Unicorn were used to generate reference vectors but are not required to run the checked-in tests. There is no Node/Bun toolchain or standalone script runner.

## Testing & QA

Rust's built-in harness covers animation behavior in inline core tests and 1,000 golden comparisons in `crates/core/tests/original_x86.rs`. Add deterministic seed/time regressions beside the relevant tests for arithmetic, interlace, palette or transition changes. Preserve original reference outputs rather than regenerating them merely to accept changed behavior.

Timing/mode regressions also cover invalid time preserving future evolution, long-frame clamping in both rendering modes, and reapplying the current interlace mode without restarting the scene or fade.

Run workspace tests, then exercise the changed surface: headless export for renderer changes; an actual desktop session for input, fullscreen or window behavior. Smoke mode does not verify keyboard interactions, and headless output does not prove native UI correctness. No numerical coverage threshold or coverage tool is configured.

Golden vectors validate original integer arithmetic with a substituted floating-point tail, not complete Windows-runtime fidelity. Host sine/square-root behavior can vary across architectures; do not promise cross-platform bit-identical pixels or treat the CI matrix as proof of interactive runtime verification.
