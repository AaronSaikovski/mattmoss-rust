//! Single-instance browser bridge. JavaScript owns the clock and UI state;
//! rendering and animation remain in the shared, dependency-free Rust core.
use mattmoss_core::SmoothAnimation;
use std::cell::RefCell;

thread_local! {
    static ANIMATION: RefCell<Option<SmoothAnimation>> = const { RefCell::new(None) };
}

fn with_animation<T>(f: impl FnOnce(&mut SmoothAnimation) -> T) -> T {
    ANIMATION.with(|state| {
        f(state
            .borrow_mut()
            .as_mut()
            .expect("call init before rendering"))
    })
}

/// Start or restart a session. Invalidates any previously returned pixel pointer.
#[no_mangle]
pub extern "C" fn init(seed: u32) {
    ANIMATION.with(|state| *state.borrow_mut() = Some(SmoothAnimation::new(seed)));
}

#[no_mangle]
pub extern "C" fn advance(seconds: f64) {
    with_animation(|animation| animation.advance(seconds));
}

/// Borrow the RGBA8 frame (640 × 480 × 4 bytes) until the next mutating call.
/// JavaScript must also refresh its view whenever WebAssembly memory grows.
#[no_mangle]
pub extern "C" fn pixels() -> *const u8 {
    with_animation(|animation| animation.pixels().as_ptr())
}

#[no_mangle]
pub extern "C" fn new_pattern() {
    with_animation(SmoothAnimation::new_pattern);
}

#[no_mangle]
pub extern "C" fn set_interlaced(enabled: u32) {
    with_animation(|animation| animation.set_interlaced(enabled != 0));
}

#[no_mangle]
pub extern "C" fn scene() -> u32 {
    with_animation(|animation| animation.scene() as u32)
}
