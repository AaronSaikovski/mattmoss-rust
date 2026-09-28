//! Headless rendering: cargo run -p mattmoss-core --example render --release -- preview.ppm
use mattmoss_core::{SmoothAnimation as Animation, HEIGHT, WIDTH};
use std::{
    fs::File,
    io::{BufWriter, Write},
};
fn main() -> std::io::Result<()> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "preview.ppm".into());
    let mut animation = Animation::new(1996);
    // Finish the startup fade (one simulation second / four seconds at 0.25x).
    for _ in 0..4 {
        animation.advance(0.25);
    }
    let mut output = BufWriter::new(File::create(&path)?);
    writeln!(output, "P6\n{} {}\n255", WIDTH, HEIGHT)?;
    for pixel in animation.pixels().as_chunks::<4>().0 {
        output.write_all(&pixel[..3])?;
    }
    output.flush()?;
    println!("Rendered {path}");
    Ok(())
}
