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
import { Scene, lookAt } from './scene.js';
import { Mat4 } from './mathlib.js';
import { EFFECTS } from './effects.js';

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

// Hand the main thread back to the renderer. Chains of `await` only
// drain the microtask queue, so without a real task boundary a phone
// runs the whole load with nothing repainted and the status line frozen
// on whatever it said first.
function breathe() {
  return new Promise((r) => setTimeout(r, 0));
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

  // Straight from the decoded image element. Going via a 2D canvas and
  // getImageData would work too, but it allocates a full backing store
  // per image and iOS caps how much canvas memory a page may hold — with
  // eighty-odd images, some of them 2048 wide, that is the difference
  // between loading and not.
  async texture(rec) {
    if (this.tex.has(rec.file)) return this.tex.get(rec.file);
    const p = (async () => {
      const img = new Image();
      img.src = this.base + rec.file;
      if (img.decode) await img.decode();
      else await new Promise((ok, no) => { img.onload = ok; img.onerror = no; });
      return this.mgl.createTextureFromImage(img, false, true);
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
  }

  async load() {
    this.status('initialising WebGL…');
    await breathe();
    this.mgl = new MiniGL(this.canvas);
    this.assets = new Assets(this.mgl, this.base);

    this.status('loading the demo script…');
    this.script = await (await fetch(this.base + 'timeline.json')).json();

    this.textures = new Map();
    const images = Object.entries(this.script.images);
    let n = 0;
    for (const [name, rec] of images) {
      this.status('loading artwork… ' + ++n + '/' + images.length);
      await breathe();
      try {
        this.textures.set(name, { tex: await this.assets.texture(rec), rec });
      } catch (e) {
        // ptr_und_ul.jpg and ptr_up_ul.jpg are named by the script but
        // were never shipped in the release
      }
    }

    this.scenes = new Map();
    this.sceneTex = new Map();
    const names = new Set();
    for (const inst of this.script.instances) {
      if (inst.command === 'drawscene' && inst.args[0]) names.add(String(inst.args[0]).toLowerCase());
    }
    let sn = 0;
    for (const n of names) {
      this.status('loading the scenes… ' + ++sn + '/' + names.size);
      await breathe();
      try {
        const file = n.replace(/\.i3d$/, '') + '.json';
        const json = await (await fetch(this.base + 'scenes/' + file)).json();
        const scene = new Scene(json);
        for (const mat of scene.materials) {
          for (const key of ['base', 'env']) {
            const f = mat[key];
            if (f && !this.sceneTex.has(f)) {
              this.sceneTex.set(f, await this.assets.texture({ file: 'textures/' + f })
                .catch(() => null));
            }
          }
        }
        this.scenes.set(n, scene);
      } catch (e) { /* a scene the release does not ship */ }
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
  //
  // `none` means blending off, not "inherit". render() ends with
  // glDisable(GL_BLEND) (0x10002d7a), so every bitmap starts from a clean
  // state and an unblended image is genuinely opaque — it does not pick up
  // the mode of whatever was drawn on the layer below it.
  _blend(mode, masked) {
    const mgl = this.mgl, gl = mgl.gl;
    if (masked) mode = 'mask';
    let f = null;
    if (mode === 'add') f = [gl.ONE, gl.ONE];
    else if (mode === 'mul') f = [gl.DST_COLOR, gl.ZERO];
    else if (mode === 'mask' || mode === 'alpha') f = [gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA];
    if (!f) { mgl.enableBlend(false); return; }
    mgl.blendFunc(f[0], f[1]);
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
    mgl.enableBlend(true);
    mgl.begin(mgl.TRIANGLES);
    const quad = [[0, 0], [SCREEN_W, 0], [SCREEN_W, SCREEN_H], [0, SCREEN_H]];
    for (const i of [0, 1, 2, 0, 2, 3]) {
      mgl.color4(r / 255, g / 255, b / 255, a / 255);
      mgl.vertex3(quad[i][0], quad[i][1], 0);
    }
    mgl.end();
  }

  // GL_SPHERE_MAP, computed here because minigl has no fixed-function
  // texgen. Energy3D imports glTexGeni and every scene material but the
  // credits screens leans on a reflection map, so this is the look of most
  // of the demo's 3D.
  _sphereMap(mesh, mv) {
    const n = mesh.positions.length / 3;
    if (!mesh.envUV || mesh.envUV.length !== n * 2) mesh.envUV = new Float32Array(n * 2);
    const p = mesh.positions, nm = mesh.normals, out = mesh.envUV, m = mv.m;
    for (let i = 0; i < n; i++) {
      const x = p[i * 3], y = p[i * 3 + 1], z = p[i * 3 + 2];
      // eye-space position and normal (the transform is rigid, so the
      // normal takes the same rotation)
      let ex = m[0] * x + m[4] * y + m[8] * z + m[12];
      let ey = m[1] * x + m[5] * y + m[9] * z + m[13];
      let ez = m[2] * x + m[6] * y + m[10] * z + m[14];
      const el = Math.hypot(ex, ey, ez) || 1;
      ex /= el; ey /= el; ez /= el;
      const ax = nm[i * 3], ay = nm[i * 3 + 1], az = nm[i * 3 + 2];
      let nx = m[0] * ax + m[4] * ay + m[8] * az;
      let ny = m[1] * ax + m[5] * ay + m[9] * az;
      let nz = m[2] * ax + m[6] * ay + m[10] * az;
      const nl = Math.hypot(nx, ny, nz) || 1;
      nx /= nl; ny /= nl; nz /= nl;
      const d = 2 * (nx * ex + ny * ey + nz * ez);
      const rx = ex - d * nx, ry = ey - d * ny, rz = ez - d * nz;
      const q = 2 * Math.sqrt(rx * rx + ry * ry + (rz + 1) * (rz + 1)) || 1;
      out[i * 2] = rx / q + 0.5;
      out[i * 2 + 1] = ry / q + 0.5;
    }
    return out;
  }

  _drawScene(inst, p, time) {
    const scene = this.scenes.get(String(inst.args[0] || '').toLowerCase());
    if (!scene) return;
    const mgl = this.mgl, gl = mgl.gl;
    const offset = parseFloat(inst.args[3]) || 0;
    const frame = scene.frameAt((time - inst.start) * 1000, offset);
    scene.update(frame);
    const cam = scene.cameraAt(inst.args[1], frame);
    if (!cam) return;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadMatrix(scene.projection(cam.fov));
    const view = lookAt(cam.eye, cam.at, [0, 1, 0]);

    mgl.matrixMode(mgl.MODELVIEW);
    // the write mask has to be back on before the clear, or the previous
    // layer's depth survives and occludes this scene
    mgl.depthMask(true);
    gl.clear(gl.DEPTH_BUFFER_BIT);
    mgl.enableDepthTest(true);
    mgl.enableCullFace(false);

    // drawScene::run never touches the colour — it only passes alpha
    // through set_alpha — so the tint comes from the material, not the
    // script.
    const a = p.alpha[0] / 255;
    if (a < 0.999) {
      mgl.enableBlend(true);
      mgl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
      mgl.depthMask(false);
    } else {
      mgl.enableBlend(false);
    }

    for (const mesh of scene.meshes) {
      const mv = new Mat4();
      mv.copy(view);
      mv.mult(mesh.world);
      mgl.loadMatrix(mv);

      const mat = scene.materials[mesh.src.material] || null;
      let uvs = null, tex = null;
      if (mat && mat.base && mesh.uvs) {
        tex = this.sceneTex.get(mat.base); uvs = mesh.uvs;
      } else if (mat && mat.env) {
        tex = this.sceneTex.get(mat.env);
        if (tex) uvs = this._sphereMap(mesh, mv);
      } else if (mat && mat.base) {
        tex = this.sceneTex.get(mat.base); uvs = mesh.uvs || mesh.zeroUV;
      }
      // ogl_material, 0x100054ac and 0x100054e7: a textured mesh draws
      // white and lets the texture carry the colour, its alpha scaled by
      // the material's opacity; an untextured one draws in the material's
      // own colour. With lighting off — which it is for every scene but
      // bolletje — that glColor4f is the only colour these meshes get.
      const op = mat ? (mat.opacity === undefined ? 1 : mat.opacity) : 1;
      if (tex || !mat || !mat.color) mgl.color4(1, 1, 1, a * op);
      else mgl.color4(mat.color[0], mat.color[1], mat.color[2], a * op);
      mgl.enableTexture(!!tex);
      if (tex) mgl.bindTexture(tex);
      if (!uvs) uvs = mesh.zeroUV;
      mgl.drawElements(mesh.positions, uvs, mesh.indices);
    }

    mgl.enableDepthTest(false);
    mgl.depthMask(false);
  }

  // One of xotrack's show* tasks. They keep per-instance state, so each
  // script instance gets its own object, built the first time it is seen.
  _drawEffect(inst, p, time) {
    if (!this._fx) this._fx = new Map();
    let fx = this._fx.get(inst);
    if (!fx) {
      fx = new (EFFECTS[inst.command])(inst);
      this._fx.set(inst, fx);
    }
    this._setup2d();
    fx.draw(this.mgl, (time - inst.start) * 1000, p);
  }

  renderFrame(time) {
    const mgl = this.mgl;
    mgl.clear();
    const active = [];
    const live = this.script.instances.filter(
      (i) => time >= i.start && time < i.end);
    live.sort((a, b) => (a.layer - b.layer) || (a.start - b.start));

    for (const inst of live) {
      const done = inst.command === 'drawimage' || inst.command === 'colorfade' ||
        (inst.command === 'drawscene' && this.scenes.has(String(inst.args[0] || '').toLowerCase())) ||
        !!EFFECTS[inst.command];
      active.push({ layer: inst.layer, command: inst.command,
                    what: inst.args[0] || '', done });
      if (!done) continue;
      const p = this._props(inst, time);
      try {
        if (inst.command === 'drawimage') this._drawImage(inst, p);
        else if (inst.command === 'drawscene') this._drawScene(inst, p, time);
        else if (inst.command === 'colorfade') this._colorFade(inst, p);
        else this._drawEffect(inst, p, time);
      } catch (e) { /* keep the loop alive */ }
    }
    this.onActiveChange(time, active);
  }
}
