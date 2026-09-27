# Validation

## Completed locally

- Linux x86_64 release executable compiled successfully using Rust 1.98.1 and Macroquad 0.4.14. Cargo.lock is included.
- `cargo test --workspace`: all six test functions passed, including 1,000 comparisons against reference outputs from the original x86 integer routine.
- Default first-frame and subsequent-frame coverage of every row, opt-in alternating-row coverage, complete-image coverage, palette changes over time and repeatable seeded sessions passed.
- The pure Rust headless renderer executed successfully and generated `preview.png`; its output was visually inspected.
- `cargo fmt --all --check` passed.

- New smoothing tests verify startup from black, continuous first transition frames, pause-safe rendering, completed crossfades, and fractional palette movement without a discrete timer step.

## Not yet verified

- Native desktop window/input/fullscreen behaviour could not be exercised in this environment: no desktop display was available and the attempted virtual display could not create its listening sockets.
- macOS and Windows builds/runs have not been performed locally. The included CI workflow is configured for native builds/tests on all three operating systems but has not been executed here.
- No original-versus-port side-by-side Windows screenshot comparison was performed.
- No signed macOS app bundle, Windows installer, or system-screensaver registration is provided.

## Included executable

`dist/linux-x86_64/mattmoss` is the locally compiled Linux build. It needs a normal desktop display and working OpenGL drivers. It is not a macOS or Windows binary. Build on your own OS using `cargo run --release --locked`.

## Fidelity scope

The reference test harness runs the original integer instructions in Unicorn, with host-supplied square-root/conversion results. It checks the integer field maths, not the full Windows runtime or the original floating-point library. See REVERSE_ENGINEERING.md.
