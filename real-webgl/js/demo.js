// "We Ain't Real" port runtime — plays the demo's own script.
//
// `real.scp` is the whole timeline in text, and it documents its own
// grammar. The build step turns it into instances: a `drawImage`,
// `drawScene`, `colorFade` or `show*` command claims a layer, runs until
// `stopAll` clears it, and carries keyframe tracks for whatever the
// script animated on it — alpha, colour, translation, rotation, scale.
//
// The engine is Red v2.6.0 by Druid / Nostalgia, built 30 Sep 2000, on
// top of Energy3D. Both DLLs ship with MSVC decorated names intact, so
// the whole 2D path here is transcribed rather than guessed:
//
//   red_base::demo_loop         glDisable(CULL_FACE / DEPTH_TEST / LIGHTING)
//                               glOrtho(0, w-1, h-1, 0, 0, 100)
//   drawImage::run              position, angle, rgb, scale, alpha, draw
//   ogl_bitmap::render          translate(x,y) · translate(w/2,h/2) ·
//                               rotate(angle,z) · scale(sw,sh) ·
//                               translate(-w/2,-h/2)
//   colorFade::run              a screen quad in glColor4ub, skipped at alpha 0
//   spline_tcb::Ease            the keyframe ease, below
//
// The clock is the shipped soundtrack, a plain MP3, as the original's
// was through BASS.

import { MiniGL } from './minigl.js';

const SCREEN_W = 640, SCREEN_H = 480;

// Defaults for a property with no track, and for the stretch of time
// before its first keyframe — Red leaves the message field untouched
// until a key is reached, so the value is whatever the message was
// constructed with.
const DEFAULTS = {
  alpha: [255],
  color: [255, 255, 255],
  translation: [0, 0, 0],
  rotation: [0, 0, 0],
  scale: [1, 1, 1],
};

// spline_tcb::Ease, transcribed from Energy3D.dll at 0x1000e950.
//
// `a` and `b` are the ease-to and ease-from terms, and the disassembly
// settles a question the script alone can't: they are loaded with `fild`,
// so they are integers taken raw, not percentages. Whenever a + b > 1
// they are renormalised to sum to 1, which is why this script's 25/25 and
// 150/150 pairs — the only non-zero values it uses — produce exactly the
// same curve.
function ease(u, a, b) {
  const s = a + b;
  if (s === 0) return u;
  if (s > 1) { a /= s; b /= s; }
  const k = 1 / (2 - a - b);
  if (u < a) return k * u * u / a;
  if (u < 1 - b) return k * (2 * u - a);
  const w = 1 - u;
  return 1 - k * w * w / b;
}

// Walk to the last key at or before `time`, then interpolate towards the
// next one. Red holds the final value after the last key, and leaves the
// property at its default before the first.
function track(keys, prop, time, out) {
  if (!keys || !keys.length || time < keys[0].t) return DEFAULTS[prop];
  let i = 0;
  while (i < keys.length - 1 && keys[i + 1].t <= time) i++;
  if (i >= keys.length - 1) return keys[i].v;
  const k0 = keys[i], k1 = keys[i + 1];
  const span = (k1.t - k0.t) || 1e-6;
  const u = ease((time - k0.t) / span, k0.easeTo, k0.easeFrom);
  for (let j = 0; j < k0.v.length; j++) {
    out[j] = k0.v[j] + (k1.v[j] - k0.v[j]) * u;
  }
  out.length = k0.v.length;
  return out;
}

class Assets {
  constructor(mgl, base) {
    this.mgl = mgl;
    this.base = base;
    this.tex = new Map();
  }

  async texture(rec) {
    if (this.tex.has(rec.file)) return this.tex.get(rec.file);
    const p = (async () => {
      const img = new Image();
      img.src = this.base + rec.file;
      await img.decode();
      const cv = document.createElement('canvas');
      cv.width = img.naturalWidth; cv.height = img.naturalHeight;
      const ctx = cv.getContext('2d', { willReadFrequently: true });
      ctx.drawImage(img, 0, 0);
      const id = ctx.getImageData(0, 0, cv.width, cv.height);
      return this.mgl.createTextureFromData(
        new Uint8Array(id.data.buffer), cv.width, cv.height, false, true);
    })();
    this.tex.set(rec.file, p);
    return p;
  }
}

export class Demo {
  constructor(canvas, status = () => {}, opts = {}) {
    this.canvas = canvas;
    this.status = status;
    this.opts = opts;
    this.base = opts.base || 'data/';
    this.running = false;
    this.timeOverride = null;
    this.onActiveChange = opts.onActiveChange || (() => {});
    this._v = [0, 0, 0];
    // Blend state persists between draws exactly as it does in the
    // original: ogl_bitmap::render only touches glBlendFunc/glEnable when
    // the image declares a mode, so a `none` image inherits whatever the
    // last blended one left behind.
    this._blendState = null;
  }

  async load() {
    this.status('initialising WebGL…');
    this.mgl = new MiniGL(this.canvas);
    this.assets = new Assets(this.mgl, this.base);

    this.status('loading the demo script…');
    this.script = await (await fetch(this.base + 'timeline.json')).json();

    this.status('loading artwork…');
    this.textures = new Map();
    for (const [name, rec] of Object.entries(this.script.images)) {
      try {
        this.textures.set(name, { tex: await this.assets.texture(rec), rec });
      } catch (e) {
        // ptr_und_ul.jpg and ptr_up_ul.jpg are named by the script but
        // were never shipped in the release
      }
    }

    this.status('loading the soundtrack…');
    this.audio = new Audio(this.base + 'real.mp3');
    this.audio.preload = 'auto';
    this.status('ready');
  }

  start() {
    if (this.running) return;
    this.running = true;
    if (this.audio) { this.audio.currentTime = 0; this.audio.play().catch(() => {}); }
    this.startClock = performance.now() / 1000;
    const frame = () => {
      if (!this.running) return;
      this.renderFrame(this.getTime());
      requestAnimationFrame(frame);
    };
    requestAnimationFrame(frame);
  }

  stop() {
    this.running = false;
    if (this.audio) this.audio.pause();
  }

  seek(t) {
    t = Math.max(0, Math.min(this.script.length, t));
    if (this.audio) this.audio.currentTime = t;
    this.startClock = performance.now() / 1000 - t;
  }

  getTime() {
    if (this.timeOverride !== null) return this.timeOverride;
    if (this.audio && !this.audio.paused && this.audio.currentTime > 0) return this.audio.currentTime;
    return performance.now() / 1000 - this.startClock;
  }

  _props(inst, time) {
    const p = {};
    for (const k of Object.keys(DEFAULTS)) {
      p[k] = track(inst.tracks[k], k, time, this._v.slice());
    }
    // colour and alpha are integers in the message — the keys hold ints
    // and the interpolated result goes through ftol before glColor4ub
    p.color = p.color.map((c) => Math.max(0, Math.min(255, c | 0)));
    p.alpha = [Math.max(0, Math.min(255, p.alpha[0] | 0))];
    return p;
  }

  // red_base::demo_loop sets this once and never changes it: no culling,
  // no depth test, and an ortho frame in screen pixels with the origin
  // top-left.
  _setup2d() {
    const mgl = this.mgl;
    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.ortho(0, SCREEN_W - 1, SCREEN_H - 1, 0, 0, 100);
    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);
  }

  // The four modes `loadImage` accepts, as ogl_bitmap::render maps them.
  // A 32-bit image — one given a separate opacity JPEG — always takes the
  // alpha path whatever it declared.
  _blend(mode, masked) {
    const mgl = this.mgl, gl = mgl.gl;
    if (masked) mode = 'mask';
    let f = null;
    if (mode === 'add') f = [gl.ONE, gl.ONE];
    else if (mode === 'mul') f = [gl.DST_COLOR, gl.ZERO];
    else if (mode === 'mask' || mode === 'alpha') f = [gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA];
    if (!f) return;          // `none`: leave the state alone, as the original does
    if (!this._blendState || this._blendState[0] !== f[0] || this._blendState[1] !== f[1]) {
      mgl.blendFunc(f[0], f[1]);
      this._blendState = f;
    }
    mgl.enableBlend(true);
  }

  _drawImage(inst, p) {
    const entry = this.textures.get(String(inst.args[0] || '').toLowerCase());
    if (!entry) return;
    const mgl = this.mgl;
    const [w, h] = entry.rec.size;
    const [sx, sy] = p.scale;
    const rot = p.rotation[2] || 0;

    // The second parameter is a flag: when it is "1" the bitmap's x is
    // replaced by (screen width - image width) / 2 and the animated
    // translation x is ignored. y is never centred.
    const tx = String(inst.args[1] || '') === '1'
      ? (SCREEN_W - w) / 2 : p.translation[0];
    const ty = p.translation[1];

    this._setup2d();
    this._blend(entry.rec.mode, entry.rec.mask);
    mgl.enableTexture(true);
    mgl.bindTexture(entry.tex);

    mgl.translate(tx, ty, 0);
    mgl.translate(w / 2, h / 2, 0);
    if (rot) mgl.rotate(rot, 0, 0, 1);
    if (sx !== 1 || sy !== 1) mgl.scale(sx, sy, 1);
    mgl.translate(-w / 2, -h / 2, 0);

    const [r, g, b] = p.color;
    const a = p.alpha[0] / 255;
    const quad = [[0, 0, 0, 0], [w, 0, 1, 0], [w, h, 1, 1], [0, h, 0, 1]];
    mgl.begin(mgl.TRIANGLES);
    for (const i of [0, 1, 2, 0, 2, 3]) {
      const [x, y, u, v] = quad[i];
      mgl.color4(r / 255, g / 255, b / 255, a);
      mgl.texCoord2(u, v);
      mgl.vertex3(x, y, 0);
    }
    mgl.end();
  }

  _colorFade(inst, p) {
    const mgl = this.mgl, gl = mgl.gl;
    const a = p.alpha[0];
    if (a === 0) return;
    const [r, g, b] = p.color;
    this._setup2d();
    mgl.enableTexture(false);
    mgl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    this._blendState = [gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA];
    mgl.enableBlend(true);
    mgl.begin(mgl.TRIANGLES);
    const quad = [[0, 0], [SCREEN_W, 0], [SCREEN_W, SCREEN_H], [0, SCREEN_H]];
    for (const i of [0, 1, 2, 0, 2, 3]) {
      mgl.color4(r / 255, g / 255, b / 255, a / 255);
      mgl.vertex3(quad[i][0], quad[i][1], 0);
    }
    mgl.end();
  }

  renderFrame(time) {
    const mgl = this.mgl;
    mgl.clear();
    const active = [];
    const live = this.script.instances.filter(
      (i) => time >= i.start && time < i.end);
    live.sort((a, b) => (a.layer - b.layer) || (a.start - b.start));

    for (const inst of live) {
      const done = inst.command === 'drawimage' || inst.command === 'colorfade';
      active.push({ layer: inst.layer, command: inst.command,
                    what: inst.args[0] || '', done });
      if (!done) continue;
      const p = this._props(inst, time);
      try {
        if (inst.command === 'drawimage') this._drawImage(inst, p);
        else this._colorFade(inst, p);
      } catch (e) { /* keep the loop alive */ }
    }
    this.onActiveChange(time, active);
  }
}
