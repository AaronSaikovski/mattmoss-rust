# Repository Guidelines

## Project Overview

Rust reconstruction of the 1996 Mattmoss Windows screensaver, delivered as a standalone desktop animation for macOS, Windows and Linux and a static browser experience using WebAssembly. This is reconstructed code, not recovered original source or OS screensaver integration. Preserve the distinction between historical arithmetic fidelity and intentionally modern presentation.

## Architecture & Data Flow

- `mattmoss-core` is dependency-free, synchronous CPU rendering. `field` implements the recovered integer kernel; `Animation` owns seeded RNG, scatter traversal, palette/timer state and reusable pixel buffers. `SmoothAnimation` wraps it with palette interpolation and scene crossfades.
- Desktop imports **`SmoothAnimation as Animation`**. Inputs/elapsed time → animation state → borrowed 640×480 RGBA8 buffer → reusable Macroquad image/texture → letterboxed window. Keep clock, keyboard, window and GPU concerns in desktop.
- `crates/web` exposes a single `SmoothAnimation` through a small WASM ABI; `web/app.js` owns browser controls/timing and copies RGBA into Canvas 2D. Refresh pixel views after memory growth or `init`; the core owns all rendering maths. Browser playback is opt-in and suspends in hidden tabs.
- The headless example consumes the same smooth renderer and writes RGB P6 PPM without a display. Default full-image mode prepares complete fields; interlaced mode retains progressive historical drawing.
- Core state is owned by animation structs; desktop state stays local, while the WASM bridge retains one thread-local animation instance. Inject a seed through `new(seed)` and simulation time through `advance(seconds)`; no service container. Rust async is limited to Macroquad's `next_frame().await`; browser loading uses promises and playback uses requestAnimationFrame.

## Key Directories

- `crates/core/src/`: numerical kernel, both animation implementations and inline unit tests.
- `crates/core/tests/`: embedded original-x86 golden vectors; inspect relevant cases rather than loading the entire large fixture file.
- `crates/core/examples/`: headless export utility, not a separate scripting framework.
- `crates/desktop/src/`: native CLI, controls and rendering loop.
- `crates/web/src/`: Rust WASM bridge; `web/`: vanilla HTML/CSS/ES-module landing page and player.
- `scripts/build-web.py`: Python standard-library build/assembly into ignored `dist/web/`.
- `.github/workflows/`: native build/test matrix and executable artifact uploads.

## Development Commands

Run from the repository root:

```sh
cargo build --release --locked
cargo run --release --locked -- --seed 1996
cargo test --workspace --locked
cargo test -p mattmoss-core --test original_x86 --locked
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo run -p mattmoss-core --example render --release --locked -- preview.ppm
cargo run --release --locked -- --seed 1996 --smoke-test
rustup target add wasm32-unknown-unknown
python3 scripts/build-web.py
python3 -m http.server 8080 --bind 127.0.0.1 --directory dist/web
```

The workspace defaults to **desktop only**: use `--workspace` for the complete test suite. Formatting and warning-free Clippy are required local and CI gates. The desktop smoke flag exits after 180 frames and still requires a working graphical display; use the core example for headless execution. The exporter writes PPM regardless of the output extension.

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
- `.github/workflows/build.yml`: stable-Rust formatting/strict Clippy, tests and release builds on Linux, Windows and macOS, plus a static WASM-site artifact.
- `.github/workflows/deploy-wasm.yml`: reusable artifact-only deployment, called by `build.yml` on `main` after both native and web jobs pass. Build/upload the Pages artifact once in the web job; do not rebuild or rerun tests in deployment. Keep Pages/OIDC write permissions confined to the deployment call/job.
- `.github/workflows/release-desktop.yml`: `v*` tags build/test Linux x86-64, Windows x86-64 and both macOS architectures, then publish archives only after all jobs pass. Keep release write permissions limited to the publish job; preserve Unix executable modes.
- `CHANGELOG.md`: record user-visible additions, fixes and tooling/deployment changes under `Unreleased`; do not invent release versions or dates.
- Read `REVERSE_ENGINEERING.md` before changing field arithmetic, RNG, palette or historical timing: it records reconstruction evidence and intentional adaptations.
- Read `README.md` for user-facing commands/controls and `VALIDATION.md` for historical verification limits. Historical pass claims are not fresh results. `preview.png` is a reference asset, not an automated golden-image fixture.

## Runtime/Tooling Preferences

Use stable Rust and Cargo (edition 2021); no numeric MSRV or pinned toolchain is declared. Keep `--locked` for normal builds/tests and retain the committed lockfile. Desktop pins Macroquad exactly to `0.4.14` with default features disabled; preserve that policy unless intentionally updating dependencies.

Native builds need a platform linker: Xcode command-line tools on macOS, MSVC/Visual Studio C++ tools on Windows, or a C linker on Linux. GUI execution also needs a desktop graphics environment (OpenGL support on Linux). Browser builds need the rustup `wasm32-unknown-unknown` target and Python 3; serve the entire generated directory, including the bundled logos, over HTTP(S), not `file://`. Read README's browser troubleshooting if Homebrew Rust shadows rustup or macOS `rust-lld` cannot find LLVM. No Node/Bun, bundler, wasm-bindgen or CDN is used. Unicorn was used to generate reference vectors but is not required to run tests.

## Testing & QA

Before completing a change, run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets --locked -- -D warnings`; both must pass. If formatting fails, run `cargo fmt --all`, then rerun the checks. Fix Clippy findings rather than weakening the CI gate or adding broad lint suppressions. Retain the narrow, documented `clippy::approx_constant` allowance for the historical `3.14` constant. Run `cargo test --workspace --locked` after Rust changes and the relevant runtime smoke check. Report environmental blockers explicitly instead of claiming an unrun check passed.

Every new feature MUST update both `README.md` and `CHANGELOG.md` in the same change before completion. Document user-facing behaviour, commands/setup and relevant limitations in the README; add a concise entry under `Unreleased` in the changelog. Feature code, verification and both documentation updates form one deliverable; do not defer the documentation.

Rust's built-in harness covers animation behavior in inline core tests and 1,000 golden comparisons in `crates/core/tests/original_x86.rs`. Add deterministic seed/time regressions beside the relevant tests for arithmetic, interlace, palette or transition changes. Preserve original reference outputs rather than regenerating them merely to accept changed behavior.

Timing/mode regressions also cover invalid time preserving future evolution, long-frame clamping in both rendering modes, and reapplying the current interlace mode without restarting the scene or fade.

Run workspace tests, then exercise the changed surface: headless export for renderer changes; an actual desktop session for input, fullscreen or window behavior. Smoke mode does not verify keyboard interactions, and headless output does not prove native UI correctness. No numerical coverage threshold or coverage tool is configured.

For browser changes, rebuild the site and exercise actual WASM animation, pause, seed restart, interlace, speed and fullscreen in a browser. Check mobile layout, reduced motion and failed-download retry; native tests do not cover the JavaScript/WASM boundary.

Golden vectors validate original integer arithmetic with a substituted floating-point tail, not complete Windows-runtime fidelity. Host sine/square-root behavior can vary across architectures; do not promise cross-platform bit-identical pixels or treat the CI matrix as proof of interactive runtime verification.
