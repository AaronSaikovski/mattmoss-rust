# Mattmoss — Rust desktop and WebAssembly recreation

A working animation reconstructed from the supplied 1996 **Mattmoss Screen Saver 1.0.0.1**, originally © HyperDyne Pty Ltd Australia. The shared Rust renderer runs in a Macroquad desktop app or as WebAssembly in a browser, with a responsive landing page and interactive canvas.

Targets: **macOS, Windows, Linux and modern browsers**. The desktop app runs locally without a browser or network after building; it is not a registered `.saver` bundle or Windows `.scr` installer.

## Run in a browser

Install stable Rust with rustup and Python 3, then build and serve the static site:

```sh
rustup target add wasm32-unknown-unknown
python3 scripts/build-web.py
python3 -m http.server 8080 --bind 127.0.0.1 --directory dist/web
```

Open **http://localhost:8080**. Use HTTP rather than opening `index.html` directly. On Windows, `py -3` can replace `python3`.

The landing page shows a real Rust-rendered still until you choose **Start the animation**. Controls provide pause/resume, new patterns, seed restart (0–4294967295), speed, historical interlacing and fullscreen where supported. With the player focused, use Space, N, R, I, F and +/−; typing in controls does not trigger shortcuts. Background tabs stop advancing, and changing to reduced motion pauses playback. Paused pattern/seed changes prepare a new still without autoplay.

The homepage's “The maths behind Mattmoss” section explains weighted sources, field magnitudes, palette cycling and smoothstep transitions, with an expandable reference for the original wrapping-integer arithmetic.

The build assembles the page, stylesheet, JavaScript, WebAssembly module and both logo images in `dist/web/`. Deploy the entire directory to any static HTTP(S) host, including under a subdirectory. No Node/Bun, bundler, CDN, backend, original executable or runtime network service is required; the browser fetches only the site's own assets. The Rust build workflow uploads a downloadable `mattmoss-web` artifact.

### GitHub Pages deployment

In the repository's **Settings → Pages → Build and deployment**, select **GitHub Actions** as the source. The **Rust builds** workflow (`.github/workflows/build.yml`) runs on pushes to `main`, pull requests, or manually from the Actions tab. It checks formatting, runs Clippy with warnings denied, tests/builds the native workspace on Linux, Windows and macOS, and builds the static site once using stable Rust with the `wasm32-unknown-unknown` target and Python 3.

On `main`, the web job also uploads that same `dist/web/` output with `actions/upload-pages-artifact`. After both the native matrix and web job succeed, it calls the reusable `.github/workflows/deploy-wasm.yml` to publish the existing artifact with `actions/deploy-pages`; deployment performs no rebuild or duplicate tests. For manual deployment, run **Rust builds** with `main` selected. The deployment's `github-pages` environment exposes the resulting URL. Without a custom domain, the expected project URL is `https://aaronsaikovski.github.io/mattmoss-rust/`. All runtime asset paths are relative, so the repository subpath requires no bundler base-URL setting. Custom-domain configuration belongs in Pages settings; this repository does not supply a `CNAME`.

Pull requests and manual runs on other branches receive the build checks and downloadable artifacts, but never upload a Pages artifact or deploy. Pages write permissions are confined to the deployment call/job. These workflows use neither Go/Ebiten, `wasm_exec.js`, nor a separate `wasm-opt` stage; Cargo's release profile supplies the Rust optimisation settings.


**Toolchain troubleshooting:** `cargo` and `rustc` must use the rustup toolchain where the WASM target was installed. If Homebrew Rust shadows it, prepend the matching binaries before building:

```sh
export PATH="$(dirname "$(rustup which rustc)"):$PATH"
```

If that macOS toolchain's `rust-lld` reports missing `@rpath/libLLVM.dylib`, the local build can use:

```sh
DYLD_LIBRARY_PATH="$(rustc --print sysroot)/lib" python3 scripts/build-web.py
```

The browser host (`crates/web`) exposes a small WASM API around `SmoothAnimation`; `web/app.js` supplies timing/input and draws the borrowed RGBA buffer using Canvas 2D. It refreshes typed views when WASM memory grows. Native and browser rendering share the same core, rather than reimplementing the maths in JavaScript.

## Run

Install stable Rust from https://rustup.rs. On macOS, install the Xcode command-line tools (`xcode-select --install`) if they are not already installed. On Windows, use the MSVC Rust toolchain with the Visual Studio C++ build tools. Linux needs a C linker and an available desktop display with OpenGL support.

Open a terminal in the extracted `mattmoss-rust` directory:

```sh
cargo run --release --locked
```

The first build downloads Rust dependencies. After that, run the compiled executable directly:

- macOS / Linux: `./target/release/mattmoss`
- Windows: `target\release\mattmoss.exe`

Optional arguments:

```sh
cargo run --release --locked -- --fullscreen
cargo run --release --locked -- --seed 1996
cargo run --release --locked -- --interlace
```

## Controls

| Key | Action |
|---|---|
| Space | Pause / resume |
| N | Generate another pattern |
| I | Toggle original alternating scanlines / complete images |
| F | Toggle full screen |
| H | Hide / show the information overlay |
| + / − | Change speed from 0.0625× to 4× (default 0.25×) |
| R | Restart the current seed |
| Escape | Leave full screen; in a window, quit |

Version 0.1.2 starts in **complete-image mode**, with every row rendered. Press **I** or pass `--interlace` to opt into the historical effect, which leaves alternate black rows initially and mixes rows from successive patterns. Press **I** again to return to complete images. The desktop defaults to **0.25× speed**: the initial image fades in over approximately **4 seconds**, and a new pattern starts about every **86 seconds**. New patterns crossfade over about **4 seconds**. Colours interpolate continuously between palette ticks. Fields are computed completely before the fade, so scattered pixels are not exposed in the default mode. The optional historical interlaced mode retains progressive scatter drawing. Pause freezes the palette and any active fade.

Rendering uses a 640 × 480 surface, scaled with linear filtering and letterboxing (nearest-neighbour filtering in optional interlaced mode). Resizing the window does not change the generated pattern.

## Project structure

- `crates/core`: dependency-free animation, original-style random generator, palette, wrapping integer field calculations, and the `SmoothAnimation` presentation layer.
- `crates/desktop`: Macroquad desktop host and keyboard controls.
- `crates/web`: dependency-free WebAssembly host around the shared core.
- `web`: static landing page, browser controls and Canvas 2D presentation.
- `scripts/build-web.py`: locked WASM build and static-site assembly.
- `crates/core/tests/original_x86.rs`: 1,000 golden cases from emulating the original field routine, with host-provided square-root conversion.
- `crates/core/examples/render.rs`: headless image export, useful for testing without a desktop.
- `.github/workflows/build.yml`: Rust formatting/lint checks, native build/test jobs and a WASM site build; uploads desktop binaries and the static site.
- `.github/workflows/deploy-wasm.yml`: reusable GitHub Pages deployment of the checked artifact; called by `build.yml` on `main`.
- `REVERSE_ENGINEERING.md`: evidence, reconstructed formula and known differences.
- `VALIDATION.md`: what was actually checked in the creation environment.

## Validate or render without a desktop

```sh
cargo test --workspace --locked
cargo run -p mattmoss-core --example render --release --locked -- preview.ppm
```

The PPM image can be opened with an image viewer that supports PPM or converted using an image editor. A sample PNG is included. A compiled Linux x86_64 executable is also included at `dist/linux-x86_64/mattmoss`; macOS and Windows users should build with Cargo on their own machine.

## Attribution and fidelity

The browser edition is presented by **WickedAILabs**, with credits to **Hyperdyne Systems**. These presentation credits are separate from the original program's historical copyright below.

The locally bundled logos come from the official site: [light logo](https://wickedailabs.com/w1.png) (`web/wickedailabs-logo.png`) and [dark logo](https://wickedailabs.com/wickedailabslogo.png) (`web/wickedailabs-logo-dark.png`). The build copies them without hotlinking or fetching assets at build time.

Original program: **Mattmoss Screen Saver**, © 1996 HyperDyne Pty Ltd Australia. This is a new reconstruction; it is not the recovered original source and does not imply permission to redistribute the original executable. The original binary is not bundled. This package preserves attribution and makes no claim to ownership of the original program.

See the reverse-engineering notes for the distinction between verified arithmetic, reconstructed behaviour and modernized display timing. Native OS screensaver integration, installers and signed application bundles are outside this first desktop version.


## How the maths creates the image

The image is a **procedural scalar field**: every pixel receives a number calculated from a few randomly positioned source points. That number chooses a palette colour. The shapes are mathematical contours; there are no stored pictures or 3D objects.

### 1. Place weighted source points

Each source has coordinates `(px, py)` and a signed weight `w` chosen from `−4, −3, −2, −1, 1, 2, 3, 4`. The first pattern uses four sources; later patterns use 3–10. Positive and negative weights reinforce or cancel contributions, creating different bends and valleys.

### 2. Evaluate each pixel

At pixel `(x, y)`, for each source:

```text
dx = px - x
dy = py - y
d  = dx² + dy²
h  = trunc(dx × dy / 2)

u += trunc(w × dx × h / d)
v += trunc(w × dy × h / d)
```

`trunc` discards the fractional part towards zero. `u` and `v` start at zero for each pixel. If `d` is zero, the original returns the previous pixel calculation's field value; the port preserves that special case.

Ignoring integer rounding and overflow, one source contributes approximately:

```text
(u, v) = (w × dx × dy / (2 × (dx² + dy²))) × (dx, dy)
```

This is a direction-dependent vector contribution. The `dx × dy` factor is zero along the source's horizontal and vertical axes and changes sign across quadrants. Adding several contributions gives complex contour shapes. It is an artistic field, not a physical gravity simulation.

Convert the accumulated vector to a scalar magnitude:

```text
magnitude = trunc(sqrt(u² + v²)) & 65535
palette_index = (magnitude / band_divisor) % 236
```

The division is integer division. The first scene uses divisor 1; subsequent scenes use 2–4. Larger divisors spread the colour bands farther apart. Modulo 236 wraps the magnitude into a repeating set of colours.

**Important implementation detail:** the original uses signed 32-bit integer arithmetic. Multiplication and addition wrap on overflow, so this Rust port explicitly uses `wrapping_mul` and `wrapping_add`. Simply rewriting the formula with floating-point numbers would produce a different image. The mathematical approximation above explains its structure, but the executable's wrapping integer operations define the actual output. A negative overflow before square root maps to zero in this reconstruction; the original CRT error path has not been independently verified.

### 3. Build the palette

The original brightness envelope has 236 entries:

```text
E(i) = trunc(16384 × (1 - sin(3.14 × i / 235)))
```

This envelope is bright near both ends and dark near the centre. The literal is **3.14**, as found in the binary, rather than a more precise value of π.

Each RGB channel is scaled by the envelope:

```text
channel(i) = base_channel × E((i + phase) % 236) / 16384
```

Moving `phase` slides bright and dark bands across the fixed field. This creates apparent flowing motion even though the source points stay still between pattern changes. Separately, the base RGB colour moves towards randomly selected colours.

### 4. Animate slowly and smoothly

The original palette advances on discrete timer steps. `SmoothAnimation` also uses the fractional time between those steps to interpolate adjacent brightness entries and base colours. This makes quarter-speed playback smoother than simply repeating each old palette frame longer.

For scene transitions, the outgoing displayed frame is retained while a complete new field is prepared. The blend uses **smoothstep**:

```text
t = clamp(elapsed / fade_duration, 0, 1)
a = t² × (3 - 2t)
output = (1 - a) × previous_frame + a × new_frame
```

The blend's rate is zero at both ends, avoiding an abrupt start or stop. At startup, `previous_frame` is black. The new frame continues its palette animation during the fade. A fade lasts one simulation second, which is four real seconds at the default `0.25×` speed. Pattern changes occur after about 21.5 simulation seconds, or 86 real seconds at the default speed. Times are approximate and assume regular rendering; the app caps long frame gaps instead of jumping ahead after a stall.

The field calculation is preserved. Fractional palette interpolation and crossfades are modern presentation additions, and are bypassed in optional historical interlace mode.

### 5. Reproduce a session

The random generator uses the original Microsoft C recurrence:

```text
state = (state × 214013 + 2531011) modulo 2³²
random = (state >> 16) & 32767
```

`--seed 1996` makes the initial scene repeatable. The same seed, controls and simulation-time inputs reproduce the session. Exact pixel equality across all CPU architectures is not guaranteed because host floating-point sine and square-root implementations may differ slightly. The original seeded itself from Windows clock ticks; this port exposes an explicit seed instead.
