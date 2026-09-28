# Changelog

## Unreleased

### Added

- Browser edition using the shared Rust renderer compiled to WebAssembly, with a responsive landing page and Canvas 2D player.
- Browser controls for playback, seed restart, new patterns, speed, interlacing and fullscreen, with keyboard/touch support, reduced-motion handling and failed-download recovery.
- Official locally bundled WickedAILabs logos, Hyperdyne Systems presentation credits, original-program attribution and prominent GitHub repository links.
- Homepage explanations of Mattmoss's origins, seed values and rendering mathematics.
- Cross-platform static-site assembly through `scripts/build-web.py`.
- Core regression tests for invalid time inputs, long-frame clamping, reapplying the current interlace mode, and completed-fade palette/transition continuity.
- Repository guidelines requiring formatting, warning-free Clippy, relevant tests and runtime verification before completion.

### Changed

- GitHub Actions now use the Rust workspace instead of unrelated Go/Ebiten build commands. CI builds/tests desktop targets on Linux, Windows and macOS and packages the WASM site.
- GitHub Pages deployment reuses the single `dist/web/` build from CI after formatting, strict Clippy, the native test/build matrix and the web build pass. Only `main` deploys; Pages and OIDC write permissions are limited to the deployment call/job.
- Fixed-width pixel/row iteration uses `as_chunks` and `as_chunks_mut`, resolving Clippy warnings without changing rendering arithmetic or allocating additional buffers.
- Completed fades copy the current smooth palette frame instead of recomputing per-channel blends, while preserving the displayed frame for the next transition.
- Headless PPM export buffers pixel writes and explicitly flushes with error propagation before reporting success.
- Ignore rules cover generated web output, Rustfmt backups and editor swap files.

### Fixed

- Native Linux/macOS CI artifacts use a tar archive to preserve executable permissions when downloaded.
