use macroquad::prelude::*;
use mattmoss_core::{SmoothAnimation as Animation, HEIGHT, WIDTH};
use std::time::{SystemTime, UNIX_EPOCH};

fn config() -> Conf {
    Conf {
        window_title: "Mattmoss — Rust desktop".into(),
        window_width: 960,
        window_height: 720,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|x| x == "--help" || x == "-h") {
        println!("Mattmoss desktop\nOptions: --seed NUMBER  --fullscreen  --interlace  --no-interlace  --smoke-test\nKeys: Space pause, N new pattern, I interlace, F full screen, H help, +/- speed, R restart, Esc leave full screen / quit.");
        return;
    }
    let mut seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u32;
    let mut fullscreen = false;
    let mut interlaced = false;
    let mut smoke = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--seed" => {
                i += 1;
                match args.get(i).and_then(|s| s.parse().ok()) {
                    Some(s) => seed = s,
                    None => {
                        eprintln!("--seed needs an unsigned 32-bit integer");
                        return;
                    }
                }
            }
            "--fullscreen" => fullscreen = true,
            "--interlace" => interlaced = true,
            "--no-interlace" => interlaced = false,
            "--smoke-test" => smoke = true,
            arg => {
                eprintln!("Unknown option: {arg}. Use --help.");
                return;
            }
        }
        i += 1;
    }
    set_fullscreen(fullscreen);
    let mut animation = Animation::new(seed);
    animation.set_interlaced(interlaced);
    let mut image = Image {
        bytes: animation.pixels().to_vec(),
        width: WIDTH as u16,
        height: HEIGHT as u16,
    };
    let texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Linear);
    let mut paused = false;
    let mut help = true;
    let mut speed: f64 = 0.25;
    let mut frames = 0;
    loop {
        if is_key_pressed(KeyCode::Escape) {
            if fullscreen {
                fullscreen = false;
                set_fullscreen(false);
            } else {
                break;
            }
        }
        if is_key_pressed(KeyCode::F) {
            fullscreen = !fullscreen;
            set_fullscreen(fullscreen);
        }
        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::H) {
            help = !help;
        }
        if is_key_pressed(KeyCode::N) {
            animation.new_pattern();
        }
        if is_key_pressed(KeyCode::I) {
            animation.set_interlaced(!animation.interlaced());
        }
        if is_key_pressed(KeyCode::R) {
            let mode = animation.interlaced();
            animation = Animation::new(seed);
            animation.set_interlaced(mode);
        }
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            speed = (speed * 2.0).min(4.0);
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            speed = (speed * 0.5).max(0.0625);
        }
        if !paused {
            animation.advance(get_frame_time() as f64 * speed);
        }
        image.bytes.copy_from_slice(animation.pixels());
        texture.set_filter(if animation.interlaced() {
            FilterMode::Nearest
        } else {
            FilterMode::Linear
        });
        texture.update(&image);
        clear_background(BLACK);
        let scale = (screen_width() / WIDTH as f32).min(screen_height() / HEIGHT as f32);
        let size = vec2(WIDTH as f32 * scale, HEIGHT as f32 * scale);
        draw_texture_ex(
            &texture,
            (screen_width() - size.x) / 2.0,
            (screen_height() - size.y) / 2.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(size),
                ..Default::default()
            },
        );
        show_mouse(help || !fullscreen);
        if help {
            draw_rectangle(
                12.0,
                12.0,
                (screen_width() - 24.0).clamp(0.0, 760.0),
                86.0,
                Color::new(0.03, 0.04, 0.06, 0.85),
            );
            draw_text(
                "MATTMOSS / 1996 RECONSTRUCTED IN RUST",
                24.0,
                35.0,
                21.0,
                WHITE,
            );
            draw_text(
                &format!(
                    "Pattern {} · {} points · {:.2}x · {} · seed {}{}",
                    animation.scene(),
                    animation.point_count(),
                    speed,
                    if animation.interlaced() {
                        "interlaced"
                    } else {
                        "full image"
                    },
                    seed,
                    if paused { " · PAUSED" } else { "" }
                ),
                24.0,
                59.0,
                17.0,
                LIGHTGRAY,
            );
            draw_text("Space pause   N new   I interlace   F fullscreen   H hide   +/- speed   R restart   Esc exit", 24.0, 82.0, 16.0, LIGHTGRAY);
        }
        next_frame().await;
        frames += 1;
        if smoke && frames >= 180 {
            break;
        }
    }
    show_mouse(true);
}
