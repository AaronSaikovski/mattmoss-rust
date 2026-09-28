const WIDTH = 640;
const HEIGHT = 480;
const FRAME_BYTES = WIDTH * HEIGHT * 4;
const $ = (id) => document.getElementById(id);
const player = $('player');
const canvas = $('canvas');
const context = canvas.getContext('2d', { alpha: false });
const stage = $('canvas-stage');
const overlay = $('player-overlay');
const startButton = $('start');
const playButton = $('play');
const newButton = $('new-pattern');
const restartButton = $('restart');
const seedInput = $('seed');
const speedInput = $('speed');
const interlaceInput = $('interlace');
const fullscreenButton = $('fullscreen');
const message = $('player-message');
const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)');
const supportsFullscreen = Boolean(document.fullscreenEnabled && player.requestFullscreen);
const rendererControls = [playButton, newButton, restartButton, seedInput, speedInput, interlaceInput];

let renderer = null;
let frame = null;
let framePointer = -1;
let ready = false;
let loading = false;
let playing = false;
let hasStarted = false;
let requestId = null;
let previousTime = null;
let currentSeed = 1996;
let currentScene = null;

function announce(text) {
  message.textContent = text;
  message.hidden = !text;
}

function stopFrameLoop() {
  if (requestId !== null) cancelAnimationFrame(requestId);
  requestId = null;
  previousTime = null;
}

function syncPlayback() {
  const state = !ready ? (loading ? 'loading' : 'error') : !hasStarted ? 'ready' : playing ? 'playing' : 'paused';
  player.dataset.state = state;
  $('player-state').textContent = ready && playing && document.hidden ? 'Tab resting' : ({ loading: 'Loading', error: 'Unavailable', ready: 'Ready when you are', playing: 'Running', paused: 'Paused' })[state];
  playButton.textContent = playing ? 'Pause' : hasStarted ? 'Resume' : 'Start animation';
  playButton.setAttribute('aria-label', playing ? 'Pause animation' : hasStarted ? 'Resume animation' : 'Start animation');
  $('motion-note').textContent = reducedMotion.matches ? 'Reduced motion: animation starts only on request.' : 'A still frame until you press start.';
  if (!ready || !playing || document.hidden) {
    stopFrameLoop();
  } else if (requestId === null) {
    requestId = requestAnimationFrame(tick);
  }
}

function draw() {
  const pointer = renderer.pixels() >>> 0;
  const buffer = renderer.memory.buffer;
  if (pointer + FRAME_BYTES > buffer.byteLength) throw new Error('The renderer returned an invalid pixel buffer.');
  // pixels() may allocate, and memory growth detaches the old typed array.
  if (!frame || frame.data.buffer !== buffer || framePointer !== pointer) {
    frame = new ImageData(new Uint8ClampedArray(buffer, pointer, FRAME_BYTES), WIDTH, HEIGHT);
    framePointer = pointer;
  }
  context.putImageData(frame, 0, 0);
  const scene = renderer.scene() >>> 0;
  if (scene !== currentScene) {
    currentScene = scene;
    $('scene-label').textContent = `Pattern ${String(scene).padStart(3, '0')}`;
  }
}

function settleStillFrame() {
  // Finish the initial/crossfade second offscreen. This is a real Rust-rendered
  // still, not autoplay: subsequent simulation waits for explicit user intent.
  for (let step = 0; step < 4; step += 1) renderer.advance(0.25);
}

function fail(error) {
  stopFrameLoop();
  ready = false;
  loading = false;
  playing = false;
  renderer = null;
  frame = null;
  stage.setAttribute('aria-busy', 'false');
  rendererControls.forEach((control) => { control.disabled = true; });
  overlay.hidden = false;
  $('overlay-kicker').textContent = 'The canvas needs a hand';
  $('overlay-title').textContent = 'Let’s try that again.';
  $('overlay-description').textContent = 'The renderer could not run. Check your connection and try again. This page needs a browser with WebAssembly support.';
  startButton.disabled = false;
  startButton.textContent = 'Retry renderer';
  const detail = error instanceof Error ? error.message : String(error);
  announce(`Renderer error: ${detail} If retry does not help, serve the built site over HTTP with mattmoss_web.wasm beside this page.`);
  syncPlayback();
}

function tick(timestamp) {
  requestId = null;
  if (!ready || !playing || document.hidden) {
    previousTime = null;
    return;
  }
  try {
    if (previousTime !== null) {
      const elapsed = Math.min((timestamp - previousTime) / 1000, 0.25);
      renderer.advance(elapsed * Number(speedInput.value));
    }
    previousTime = timestamp;
    draw();
    requestId = requestAnimationFrame(tick);
  } catch (error) {
    fail(error);
  }
}

async function loadRenderer() {
  if (loading) return;
  stopFrameLoop();
  loading = true;
  ready = false;
  playing = false;
  hasStarted = false;
  frame = null;
  currentScene = null;
  rendererControls.forEach((control) => { control.disabled = true; });
  startButton.disabled = true;
  startButton.textContent = 'Loading renderer…';
  overlay.hidden = false;
  stage.setAttribute('aria-busy', 'true');
  $('overlay-kicker').textContent = 'Rust → WebAssembly → your screen';
  $('overlay-title').textContent = 'Preparing the canvas.';
  $('overlay-description').textContent = 'Loading the renderer. No video, no pre-recorded loop.';
  announce('');
  syncPlayback();
  try {
    if (!context) throw new Error('Canvas 2D is not supported in this browser.');
    if (typeof WebAssembly === 'undefined') throw new Error('WebAssembly is not supported in this browser.');
    const response = await fetch(new URL('./mattmoss_web.wasm', import.meta.url), { cache: 'no-cache' });
    if (!response.ok) throw new Error(`Renderer download failed (HTTP ${response.status}).`);
    const { instance } = await WebAssembly.instantiate(await response.arrayBuffer(), {});
    renderer = instance.exports;
    for (const name of ['init', 'advance', 'pixels', 'new_pattern', 'set_interlaced', 'scene']) {
      if (typeof renderer[name] !== 'function') throw new Error(`The renderer is missing its ${name} export.`);
    }
    if (!(renderer.memory instanceof WebAssembly.Memory)) throw new Error('The renderer is missing its pixel memory.');
    renderer.init(currentSeed);
    renderer.set_interlaced(Number(interlaceInput.checked));
    canvas.classList.toggle('interlaced', interlaceInput.checked);
    settleStillFrame();
    draw();
    loading = false;
    ready = true;
    stage.setAttribute('aria-busy', 'false');
    rendererControls.forEach((control) => { control.disabled = false; });
    $('overlay-kicker').textContent = 'A real-time piece of 1996';
    $('overlay-title').textContent = 'Take a moment. Do nothing.';
    $('overlay-description').textContent = reducedMotion.matches ? 'Reduced motion is on. This image stays still unless you choose to animate it.' : 'A still frame, for now. Press start to let the colours wander.';
    startButton.textContent = 'Start the animation';
    startButton.disabled = false;
    syncPlayback();
  } catch (error) {
    fail(error);
  }
}

function togglePlayback() {
  if (!ready) return;
  hasStarted = true;
  playing = !playing;
  overlay.hidden = true;
  announce('');
  syncPlayback();
}

function changeRenderer(action, confirmation) {
  if (!ready) return;
  try {
    action();
    if (!playing) settleStillFrame();
    draw();
    previousTime = null;
    announce(confirmation);
  } catch (error) {
    fail(error);
  }
}

function newPattern() {
  changeRenderer(() => renderer.new_pattern(), playing ? 'A new pattern is dissolving in.' : 'New pattern. Animation remains paused.');
}

function restart() {
  seedInput.setCustomValidity('');
  const value = seedInput.value.trim();
  if (!/^\d+$/.test(value) || Number(value) > 4294967295) {
    seedInput.setCustomValidity('Enter a whole-number seed from 0 to 4294967295.');
    seedInput.reportValidity();
    return;
  }
  currentSeed = Number(value);
  seedInput.value = String(currentSeed);
  changeRenderer(() => {
    renderer.init(currentSeed);
    renderer.set_interlaced(Number(interlaceInput.checked));
  }, `Restarted seed ${currentSeed}.${playing ? '' : ' Animation remains paused.'}`);
}

function toggleInterlace() {
  canvas.classList.toggle('interlaced', interlaceInput.checked);
  changeRenderer(() => renderer.set_interlaced(Number(interlaceInput.checked)), interlaceInput.checked ? 'Historical alternating scanlines enabled. A new pattern begins.' : 'Complete-image mode enabled. A new pattern begins.');
}

async function toggleFullscreen() {
  if (!supportsFullscreen) {
    announce('Full screen is not available in this browser. You can still use every animation control here.');
    return;
  }
  try {
    if (document.fullscreenElement === player) await document.exitFullscreen();
    else await player.requestFullscreen();
    announce('');
  } catch (error) {
    announce(`Full screen was not allowed. Try the Full screen button, or check your browser’s permissions. ${error instanceof Error ? error.message : ''}`);
  }
}

startButton.addEventListener('click', () => {
  if (!ready) { void loadRenderer(); return; }
  togglePlayback();
  player.focus({ preventScroll: true });
});
playButton.addEventListener('click', togglePlayback);
newButton.addEventListener('click', newPattern);
$('seed-form').addEventListener('submit', (event) => { event.preventDefault(); restart(); });
seedInput.addEventListener('input', () => seedInput.setCustomValidity(''));
interlaceInput.addEventListener('change', toggleInterlace);
speedInput.addEventListener('change', () => {
  previousTime = null;
  announce(`Speed set to ${Number(speedInput.value)}×.`);
});
fullscreenButton.disabled = !supportsFullscreen;
if (!supportsFullscreen) {
  fullscreenButton.textContent = 'Full screen unavailable';
  fullscreenButton.setAttribute('aria-label', 'Full screen is unavailable in this browser');
  fullscreenButton.title = 'This browser does not support fullscreen for the player.';
}
fullscreenButton.addEventListener('click', () => { void toggleFullscreen(); });
document.addEventListener('fullscreenchange', () => {
  const fullscreen = document.fullscreenElement === player;
  fullscreenButton.textContent = fullscreen ? 'Exit full screen' : 'Full screen ⤢';
  fullscreenButton.setAttribute('aria-label', fullscreen ? 'Exit fullscreen' : 'Enter fullscreen');
});
document.addEventListener('visibilitychange', () => {
  previousTime = null;
  syncPlayback();
});
reducedMotion.addEventListener('change', () => {
  if (reducedMotion.matches && playing) {
    playing = false;
    announce('Paused because your reduced-motion preference changed. Choose Resume to animate again.');
  }
  syncPlayback();
});
player.addEventListener('keydown', (event) => {
  if (event.defaultPrevented || event.ctrlKey || event.metaKey || event.altKey || event.repeat) return;
  if (event.target.closest('input, select, textarea, button, a, summary, [contenteditable="true"]')) return;
  const key = event.key.toLowerCase();
  if (![' ', 'n', 'r', 'i', 'f', '+', '=', '-', '_'].includes(key)) return;
  event.preventDefault();
  if (key === 'f') { void toggleFullscreen(); return; }
  if (!ready) return;
  if (key === ' ') togglePlayback();
  else if (key === 'n') newPattern();
  else if (key === 'r') restart();
  else if (key === 'i') {
    interlaceInput.checked = !interlaceInput.checked;
    toggleInterlace();
  } else {
    const direction = key === '+' || key === '=' ? 1 : -1;
    speedInput.selectedIndex = Math.max(0, Math.min(speedInput.options.length - 1, speedInput.selectedIndex + direction));
    speedInput.dispatchEvent(new Event('change'));
  }
});

void loadRenderer();
