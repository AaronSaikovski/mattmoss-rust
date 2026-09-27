//! Rendering core reconstructed from the supplied 1996 PE32 executable.
//! All original IMUL operations explicitly wrap, including in debug builds.
pub const WIDTH: usize = 640;
pub const HEIGHT: usize = 480;
pub const COLOURS: usize = 236;
const PASSES: usize = 48 * 36;

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
    pub weight: i32,
}

/// Translation of 0x40222c; previous value is returned at a source point.
pub fn field(points: &[Point], x: i32, y: i32, previous: u16) -> u16 {
    let (mut u, mut v) = (0i32, 0i32);
    for p in points {
        let dx = p.x.wrapping_sub(x);
        let dy = p.y.wrapping_sub(y);
        let d = dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy));
        if d == 0 {
            return previous;
        }
        let half = dx.wrapping_mul(dy) / 2;
        u = u.wrapping_add(p.weight.wrapping_mul(dx).wrapping_mul(half).wrapping_div(d));
        v = v.wrapping_add(p.weight.wrapping_mul(dy).wrapping_mul(half).wrapping_div(d));
    }
    let square = u.wrapping_mul(u).wrapping_add(v.wrapping_mul(v));
    // Negative overflow fed sqrt's error path in the original; low word is zero.
    if square < 0 {
        0
    } else {
        ((square as f64).sqrt() as u32 & 0xffff) as u16
    }
}

#[derive(Clone)]
struct MsRandom(u32);
impl MsRandom {
    // Microsoft C rand(), routine 0x4030e0.
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(214013).wrapping_add(2531011);
        (self.0 >> 16) & 32767
    }
}

pub struct Animation {
    rng: MsRandom,
    order: Vec<usize>,
    indices: Vec<i16>,
    rgba: Vec<u8>,
    envelope: [i32; COLOURS],
    colour: [i32; 3],
    origin: [i32; 3],
    delta: [i32; 3],
    phase: usize,
    duration: usize,
    points: Vec<Point>,
    divisor: usize,
    cursor: usize,
    previous: u16,
    parity: usize,
    interlaced: bool,
    scene: u64,
    draw_budget: f64,
    timer_budget: f64,
    idle_ticks: usize,
}
impl Animation {
    pub fn new(seed: u32) -> Self {
        let mut rng = MsRandom(seed);
        let colour = std::array::from_fn(|_| (rng.next() % 255) as i32);
        let mut order: Vec<usize> = (0..PASSES).collect();
        for i in 0..PASSES - 1 {
            let j = i + rng.next() as usize % (PASSES - i);
            order.swap(i, j);
        }
        // 3.14 is the literal in the binary, deliberately not PI.
        #[allow(clippy::approx_constant)]
        let envelope = std::array::from_fn(|i| {
            (16384.0 - (i as f64 * 3.14 / (COLOURS - 1) as f64).sin() * 16384.0) as i32
        });
        let mut this = Self {
            rng,
            order,
            indices: vec![-1; WIDTH * HEIGHT],
            rgba: vec![0; WIDTH * HEIGHT * 4],
            envelope,
            colour,
            origin: colour,
            delta: [0; 3],
            phase: 0,
            duration: 0,
            points: Vec::new(),
            divisor: 1,
            cursor: 0,
            previous: 0,
            parity: 0,
            interlaced: false,
            scene: 0,
            draw_budget: 0.0,
            timer_budget: 0.0,
            idle_ticks: 0,
        };
        this.start_scene(true);
        this
    }
    fn start_scene(&mut self, initial: bool) {
        self.scene += 1;
        self.parity ^= 1;
        let count = if initial {
            4
        } else {
            self.rng.next() as usize % 8 + 3
        };
        self.divisor = if initial {
            1
        } else {
            self.rng.next() as usize % 3 + 2
        };
        self.points.clear();
        for _ in 0..count {
            let x = (self.rng.next() as usize % WIDTH) as i32;
            let y = (self.rng.next() as usize % HEIGHT) as i32;
            let mut weight = 0;
            while weight == 0 {
                weight = (self.rng.next() % 9) as i32 - 4;
            }
            self.points.push(Point { x, y, weight });
        }
        self.cursor = 0;
        self.idle_ticks = 0;
        self.draw_budget = 0.0;
        self.timer_budget = 0.0;
    }
    pub fn new_pattern(&mut self) {
        self.start_scene(false);
    }
    pub fn scene(&self) -> u64 {
        self.scene
    }
    pub fn point_count(&self) -> usize {
        self.points.len()
    }
    pub fn interlaced(&self) -> bool {
        self.interlaced
    }
    pub fn is_drawing(&self) -> bool {
        self.cursor < PASSES
    }
    pub fn set_interlaced(&mut self, enabled: bool) {
        if self.interlaced != enabled {
            self.interlaced = enabled;
            self.indices.fill(-1);
            self.new_pattern();
        }
    }
    fn palette_step(&mut self) {
        if self.phase >= self.duration {
            self.duration = (self.rng.next() as usize % 4 + 1) * COLOURS;
            self.phase = 0;
            self.origin = self.colour;
            for i in 0..3 {
                self.delta[i] = (self.rng.next() % 256) as i32 - self.colour[i];
            }
        }
        for i in 0..3 {
            self.colour[i] =
                self.origin[i] + self.phase as i32 * self.delta[i] / self.duration as i32;
        }
        self.phase += 1;
    }
    /// One original scatter pass through the 48 x 36 cell ordering.
    pub fn draw_pass(&mut self) {
        if !self.is_drawing() {
            return;
        }
        let value = self.order[self.cursor];
        self.cursor += 1;
        let (row, col) = (value / 48, value % 48);
        for bx in (0..WIDTH + 47).step_by(48) {
            for by in (0..HEIGHT + 35).step_by(36) {
                let y = by + (self.order[bx] + row) % 36;
                let x = bx + (self.order[by] + col) % 48;
                if x >= WIDTH || y >= HEIGHT || (self.interlaced && y % 2 == self.parity) {
                    continue;
                }
                self.previous = field(&self.points, x as i32, y as i32, self.previous);
                self.indices[y * WIDTH + x] =
                    (self.previous as usize / self.divisor % COLOURS) as i16;
            }
        }
        self.palette_step();
    }
    /// Deterministic time input; caller implements pause by not calling this.
    pub fn advance(&mut self, seconds: f64) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        let seconds = seconds.min(0.25);
        if self.is_drawing() {
            self.draw_budget += seconds * 720.0;
            let count = self.draw_budget as usize;
            self.draw_budget -= count as f64;
            for _ in 0..count.min(PASSES - self.cursor) {
                self.draw_pass();
            }
        } else {
            self.timer_budget += seconds * 100.0;
            while self.timer_budget >= 1.0 {
                self.timer_budget -= 1.0;
                self.palette_step();
                self.idle_ticks += 1;
                if self.idle_ticks > 2150 {
                    self.new_pattern();
                    break;
                }
            }
        }
    }
    /// RGBA8 image; independent of windowing, GPU or operating system APIs.
    pub fn pixels(&mut self) -> &[u8] {
        let palette: [[u8; 3]; COLOURS] = std::array::from_fn(|i| {
            let level = self.envelope[(i + self.phase) % COLOURS];
            std::array::from_fn(|c| (self.colour[c] * level / 16384) as u8)
        });
        for (i, pixel) in self.rgba.chunks_exact_mut(4).enumerate() {
            let index = self.indices[i];
            let rgb = if index < 0 {
                [0; 3]
            } else {
                palette[index as usize]
            };
            pixel[..3].copy_from_slice(&rgb);
            pixel[3] = 255;
        }
        &self.rgba
    }
    /// Interpolate palette entries and base colour between original timer ticks.
    pub fn pixels_smooth(&mut self) -> &[u8] {
        let fraction = self.timer_budget.clamp(0.0, 1.0);
        let phase = self.phase as f64 + fraction;
        let colour: [f64; 3] = std::array::from_fn(|c| {
            if self.duration == 0 {
                self.colour[c] as f64
            } else {
                self.origin[c] as f64
                    + (self.phase.saturating_sub(1) as f64 + fraction) * self.delta[c] as f64
                        / self.duration as f64
            }
        });
        let palette: [[u8; 3]; COLOURS] = std::array::from_fn(|i| {
            let position = (i as f64 + phase) % COLOURS as f64;
            let a = position.floor() as usize;
            let t = position.fract();
            let level =
                self.envelope[a] as f64 * (1.0 - t) + self.envelope[(a + 1) % COLOURS] as f64 * t;
            std::array::from_fn(|c| (colour[c] * level / 16384.0).clamp(0.0, 255.0).round() as u8)
        });
        for (i, pixel) in self.rgba.chunks_exact_mut(4).enumerate() {
            let index = self.indices[i];
            let rgb = if index < 0 {
                [0; 3]
            } else {
                palette[index as usize]
            };
            pixel[..3].copy_from_slice(&rgb);
            pixel[3] = 255;
        }
        &self.rgba
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scatter_covers_alternating_rows_then_full_frame() {
        let mut a = Animation::new(1996);
        a.set_interlaced(true);
        while a.is_drawing() {
            a.draw_pass();
        }
        assert_eq!(
            a.indices.iter().filter(|&&i| i < 0).count(),
            WIDTH * HEIGHT / 2
        );
        a.new_pattern();
        while a.is_drawing() {
            a.draw_pass();
        }
        assert!(a.indices.iter().all(|&i| i >= 0));
        a.set_interlaced(false);
        while a.is_drawing() {
            a.draw_pass();
        }
        assert!(a.indices.iter().all(|&i| (0..COLOURS as i16).contains(&i)));
    }
    #[test]
    fn default_renders_every_row_in_first_and_subsequent_scenes() {
        let mut a = Animation::new(1996);
        assert!(!a.interlaced());
        for _ in 0..2 {
            while a.is_drawing() {
                a.draw_pass();
            }
            for row in a.indices.chunks_exact(WIDTH) {
                assert!(row.iter().all(|&i| (0..COLOURS as i16).contains(&i)));
            }
            a.new_pattern();
        }
    }
    #[test]
    fn seeded_sessions_repeat_and_change_over_time() {
        let mut a = Animation::new(1996);
        let mut b = Animation::new(1996);
        for _ in 0..200 {
            a.advance(1.0 / 60.0);
            b.advance(1.0 / 60.0);
        }
        let first = a.pixels().to_vec();
        assert_eq!(first, b.pixels());
        for _ in 0..60 {
            a.advance(1.0 / 60.0);
        }
        assert_ne!(first, a.pixels());
    }
}

/// Modern desktop presentation: complete fields, fractional palette animation,
/// and a one-second simulation-time smoothstep fade (4 seconds at default 0.25x).
pub struct SmoothAnimation {
    inner: Animation,
    from: Vec<u8>,
    output: Vec<u8>,
    elapsed: f64,
}
impl SmoothAnimation {
    pub fn new(seed: u32) -> Self {
        let mut inner = Animation::new(seed);
        while inner.is_drawing() {
            inner.draw_pass();
        }
        let mut black = vec![0; WIDTH * HEIGHT * 4];
        for pixel in black.chunks_exact_mut(4) {
            pixel[3] = 255;
        }
        Self {
            inner,
            from: black.clone(),
            output: black,
            elapsed: 0.0,
        }
    }
    pub fn scene(&self) -> u64 {
        self.inner.scene()
    }
    pub fn point_count(&self) -> usize {
        self.inner.point_count()
    }
    pub fn interlaced(&self) -> bool {
        self.inner.interlaced()
    }
    fn prepare_transition(&mut self) {
        self.from.copy_from_slice(&self.output);
        self.elapsed = 0.0;
        if !self.inner.interlaced() {
            while self.inner.is_drawing() {
                self.inner.draw_pass();
            }
        }
    }
    pub fn new_pattern(&mut self) {
        self.inner.new_pattern();
        self.prepare_transition();
    }
    pub fn set_interlaced(&mut self, enabled: bool) {
        if enabled != self.inner.interlaced() {
            self.inner.set_interlaced(enabled);
            self.prepare_transition();
        }
    }
    pub fn advance(&mut self, seconds: f64) {
        if !seconds.is_finite() || seconds <= 0.0 {
            return;
        }
        let dt = seconds.min(0.25);
        let scene = self.inner.scene();
        self.inner.advance(dt);
        if self.inner.scene() != scene {
            self.prepare_transition();
        } else {
            self.elapsed = (self.elapsed + dt).min(1.0);
        }
    }
    pub fn pixels(&mut self) -> &[u8] {
        if self.inner.interlaced() {
            self.output.copy_from_slice(self.inner.pixels());
        } else {
            let t = self.elapsed.clamp(0.0, 1.0);
            let alpha = t * t * (3.0 - 2.0 * t);
            let target = self.inner.pixels_smooth();
            for (i, out) in self.output.iter_mut().enumerate() {
                *out = if i % 4 == 3 {
                    255
                } else {
                    (self.from[i] as f64 * (1.0 - alpha) + target[i] as f64 * alpha).round() as u8
                };
            }
        }
        &self.output
    }
}

#[cfg(test)]
mod smooth_tests {
    use super::*;
    #[test]
    fn startup_fades_and_pattern_change_preserves_displayed_frame() {
        let mut a = SmoothAnimation::new(1996);
        assert!(a.pixels().chunks_exact(4).all(|p| p[..3] == [0, 0, 0]));
        for _ in 0..8 {
            a.advance(0.125);
        }
        let visible = a.pixels().to_vec();
        assert!(visible.chunks_exact(4).any(|p| p[0] > 0 || p[1] > 0));
        assert_eq!(visible, a.pixels()); // Rendering alone never advances time.
        a.new_pattern();
        assert_eq!(visible, a.pixels()); // No jump on the first transition frame.
        for _ in 0..8 {
            a.advance(0.125);
        }
        assert_ne!(visible, a.pixels());
    }
    #[test]
    fn fractional_tick_changes_palette_without_discrete_step() {
        let mut a = Animation::new(1996);
        while a.is_drawing() {
            a.draw_pass();
        }
        let before = a.pixels_smooth().to_vec();
        let phase = a.phase;
        a.advance(0.005);
        assert_eq!(phase, a.phase);
        assert_ne!(before, a.pixels_smooth());
    }
}
