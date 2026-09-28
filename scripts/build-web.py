#!/usr/bin/env python3
"""Build the browser renderer and collect the static site in dist/web."""

from pathlib import Path
import shutil
import subprocess
import sys


def main():
    root = Path(__file__).resolve().parent.parent
    command = [
        "cargo", "build", "-p", "mattmoss-web", "--target",
        "wasm32-unknown-unknown", "--release", "--locked",
    ]
    try:
        subprocess.run(command, cwd=root, check=True)
    except FileNotFoundError:
        print("error: cargo was not found; install Rust and add cargo to PATH.", file=sys.stderr)
        return 1
    except subprocess.CalledProcessError as error:
        print(
            "error: WebAssembly build failed (exit {}). "
            "If the target is missing, run: rustup target add wasm32-unknown-unknown".format(
                error.returncode
            ),
            file=sys.stderr,
        )
        return 1
    except OSError as error:
        print("error: could not run cargo: {}".format(error), file=sys.stderr)
        return 1

    destination = root / "dist" / "web"
    sources = [
        root / "target" / "wasm32-unknown-unknown" / "release" / "mattmoss_web.wasm",
        root / "web" / "index.html",
        root / "web" / "styles.css",
        root / "web" / "app.js",
        root / "web" / "wickedailabs-logo.png",
        root / "web" / "wickedailabs-logo-dark.png",
    ]
    try:
        destination.mkdir(parents=True, exist_ok=True)
        for source in sources:
            shutil.copy2(source, destination / source.name)
    except OSError as error:
        print("error: could not assemble the static site: {}".format(error), file=sys.stderr)
        return 1

    print("Static site built at {}".format(destination))
    return 0


if __name__ == "__main__":
    sys.exit(main())
