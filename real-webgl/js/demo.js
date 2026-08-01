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
import { EFFECTS, EFFECT_TEXTURES } from './effects.js';

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
      // GL_REPEAT on both axes, and GL_LINEAR both ways with no mipmaps.
      // That is what `energy3d_ogl::create_texture` (0x100023e0) does when
      // its first bool is set, and every caller in the demo passes 1 for
      // it. Nothing in the port relies on clamping — no scene mesh has a
      // UV outside [0, 1] — but showTunnel's do, twice around the tube and
      // twenty times along it, and clamped they smear instead of tile.
      //
      // The second bool builds mipmaps, and every caller passes 1 for that
      // too, but MIN_FILTER is set to plain GL_LINEAR either way, so they
      // are built and never sampled. Skipped here for the same result.
      return this.mgl.createTextureFromImage(img, false, false);
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
    this.proj3d = this._proj3dDefault();
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
      // showBol and showPartiekels load sphere.i3d for themselves; the
      // script never names it.
      const fx = EFFECTS[inst.command];
      if (fx && fx.scene) names.add(fx.scene);
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

    // The textures xotrack's effects name in Real.exe's string table.
    // They live alongside the scene textures rather than in the script's
    // image list, because `loadImage` never sees them — each effect loads
    // its own in its init.
    this.fxTex = new Map();
    for (const f of EFFECT_TEXTURES) {
      this.status('loading effect artwork… ' + f);
      await breathe();
      this.fxTex.set(f, await this.assets.texture({ file: 'textures/' + f })
        .catch(() => null));
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

  // Red keeps two drawing frames and switches between them by task *kind*,
  // which every task declares when it registers (`red_base::add_task`
  // takes the kind as its second argument). `red_base::run_tasks`
  // (0x10003d10) watches for a change of kind between consecutive layers:
  //
  //   kind 2 -> kind 1   glDisable(CULL_FACE / DEPTH_TEST / LIGHTING)
  //                      PROJECTION: push, identity, glOrtho(0, w-1, h-1, 0, 0, 100)
  //                      MODELVIEW:  push, identity
  //
  //   kind 1 -> kind 2   MODELVIEW: pop.  PROJECTION: pop.
  //                      glEnable(CULL_FACE / DEPTH_TEST / LIGHTING)
  //
  // So kind 1 is the 2D pixel frame — `drawImage`, `colorFade`, and of
  // xotrack's effects only `showLines` and `show2d` — and kind 2 draws in
  // whatever 3D frame is standing. Nothing restores that frame: the two
  // effects that set a projection of their own (`showTunnel`,
  // `showPartiekels`) push and pop it, and `ogl_bitmap::render` pushes and
  // pops too, so the standing projection is always the one the last
  // `drawScene` left behind — `ogl_camera::calculate` (0x10002dc0) never
  // restores it — or, before any scene has drawn, the one
  // `energy3d_ogl::create_display` set at 0x100011ba.
  _proj3dDefault() {
    // create_display: gluPerspective(45, 4/3, 2, 5000)
    const m = new Mat4();
    const half = Math.tan(45 * Math.PI / 360) * 2;
    m.frustum(-half * (4 / 3), half * (4 / 3), -half, half, 2, 5000);
    return m;
  }

  // The entry side of a kind-2 task. Red re-enables culling, the depth
  // test and lighting here, but every one of xotrack's effects opens by
  // setting the state it wants, so only the matrices have to be right.
  _setup3d() {
    const mgl = this.mgl;
    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadMatrix(this.proj3d);
    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();
  }

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
    mgl.enableLighting(false);
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

  // One mesh with its material applied, as ogl_material does it.
  _drawMesh(scene, mesh, view, alpha, inBlendedPass) {
    const mgl = this.mgl, gl = mgl.gl;
    const mv = new Mat4();
    mv.copy(view);
    mv.mult(mesh.world);
    mgl.loadMatrix(mv);

    const mat = scene.materials[mesh.src.material] || null;
    let uvs = null, tex = null;
    if (mat && mat.base) {
      tex = this.sceneTex.get(mat.base);
      uvs = mesh.uvs || mesh.zeroUV;
    }
    // The reflection map is *not* an alternative to the base map. It goes
    // on as a second, additive contribution — see the env pass below.
    const envTex = mat && mat.env ? this.sceneTex.get(mat.env) : null;

    // Additive materials swap the blend func in place, per material
    // (0x1000548e). Source alpha then has no effect at all, which is why
    // the cubes' 40% opacity does not dim them.
    if (inBlendedPass) {
      if (mat && mat.additive) mgl.blendFunc(gl.ONE, gl.ONE);
      else mgl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
    }

    // A textured mesh draws white and lets the texture carry the colour,
    // its alpha scaled by the material's opacity; an untextured one draws
    // in the material's own colour. With lighting off — which it is for
    // every scene that carries no lights — that glColor4f is the only
    // colour these meshes get.
    const op = mat ? (mat.opacity === undefined ? 1 : mat.opacity) : 1;
    if (tex || !mat || !mat.color) mgl.color4(1, 1, 1, alpha * op);
    else mgl.color4(mat.color[0], mat.color[1], mat.color[2], alpha * op);
    mgl.enableTexture(!!tex);
    if (tex) mgl.bindTexture(tex);

    // Max's Wire checkbox: the same index list, drawn as GL_LINES
    // (0x1000559d). effect.i3d's shape is the only one in these scenes.
    const prim = mat && mat.wire ? gl.LINES : null;
    mgl.drawElements(mesh.positions, uvs || mesh.zeroUV, mesh.indices,
                     null, mesh.normals, prim);

    if (!envTex) return;

    // The reflection pass, at 0x100055ff. Energy3D has two routes to it
    // and both *add* the map on top of what the base pass already drew:
    //
    //   with GL_ARB_multitexture and GL_EXT_texture_env_add (+0x278 and
    //   +0x27c, probed at 0x100014c2 and 0x10005142), one pass — the map
    //   goes on texture unit 1 with
    //   glTexEnvf(GL_TEXTURE_ENV, GL_TEXTURE_ENV_MODE, 260.0), and 260 is
    //   GL_ADD.
    //
    //   without them, two passes — the base, then glDepthFunc(GL_EQUAL),
    //   glBlendFunc(GL_ONE, GL_ONE) and the map again over the same
    //   triangles.
    //
    // So a reflective surface is its own colour *plus* its reflection, not
    // its reflection alone. sphere.i3d's blob is mid grey under chrome,
    // not chrome on black. The port took the second route because it is
    // the one that fits a single texture unit, but it follows the first in
    // leaving the added map unmodulated: the fallback would tint it by
    // whatever colour the base pass left current, which on a GeForce-era
    // card — where both extensions were present — never happened.
    //
    // GL_NV_texgen_reflection (+0x27b) would swap GL_SPHERE_MAP for
    // GL_REFLECTION_MAP, a different and cruder mapping. Not reproduced;
    // the sphere-map route is the portable one and the one that reads
    // correctly against the artwork.
    const env = this._sphereMap(mesh, mv);
    mgl.depthFunc(gl.EQUAL);
    mgl.blendFunc(gl.ONE, gl.ONE);
    mgl.enableBlend(true);
    mgl.bindTexture(envTex);
    mgl.enableTexture(true);
    mgl.color4(alpha, alpha, alpha, 1);
    mgl.drawElements(mesh.positions, env, mesh.indices, null, mesh.normals, prim);
    mgl.depthFunc(gl.LEQUAL);
    if (!inBlendedPass) mgl.enableBlend(false);
  }

  // Lights, transformed into eye space the way glLightfv would have them
  // after gluLookAt. Position is a point light unless the scene marks it
  // a spot, in which case the direction is target - position and the
  // cutoff is halved, as ogl_light::calculate does.
  _sceneLights(scene, view, frame) {
    const m = view.m;
    const xf = (p, w) => [
      m[0] * p[0] + m[4] * p[1] + m[8] * p[2] + m[12] * w,
      m[1] * p[0] + m[5] * p[1] + m[9] * p[2] + m[13] * w,
      m[2] * p[0] + m[6] * p[1] + m[10] * p[2] + m[14] * w,
      w,
    ];
    return scene.lights.map((l) => {
      const pos = xf(l.pos, 1);
      const out = { pos, diffuse: l.diffuse };
      if (l.spot) {
        const t = xf(l.target, 1);
        const d = [t[0] - pos[0], t[1] - pos[1], t[2] - pos[2]];
        const len = Math.hypot(d[0], d[1], d[2]) || 1;
        out.spotDir = [d[0] / len, d[1] / len, d[2] / len];
        out.spotCos = Math.cos(l.cutoff * 0.5 * Math.PI / 180);
      }
      return out;
    });
  }

  _drawScene(inst, p, time) {
    const scene = this.scenes.get(String(inst.args[0] || '').toLowerCase());
    if (!scene) return;
    const offset = parseFloat(inst.args[3]) || 0;
    const frame = scene.frameAt((time - inst.start) * 1000, offset);
    scene.update(frame);
    const cam = scene.cameraAt(inst.args[1], frame);
    if (!cam) return;
    this.renderScene(scene, cam, frame, p.alpha[0] / 255);
  }

  // The body of drawScene, without the script instance around it, so
  // showBol can reach it — that effect deforms sphere.i3d's vertices and
  // then calls `energy3d_scene::draw_scene` (0x402342), which is the same
  // path any scene takes.
  renderScene(scene, cam, frame, a) {
    const mgl = this.mgl, gl = mgl.gl;

    mgl.matrixMode(mgl.PROJECTION);
    // ogl_camera::calculate does not push this, so it stays current for
    // every kind-2 task that follows — including into the next section.
    this.proj3d = scene.projection(cam.fov);
    mgl.loadMatrix(this.proj3d);
    const view = lookAt(cam.eye, cam.at, [0, 1, 0]);

    mgl.matrixMode(mgl.MODELVIEW);
    // the write mask has to be back on before the clear, or the previous
    // layer's depth survives and occludes this scene
    mgl.depthMask(true);
    gl.clear(gl.DEPTH_BUFFER_BIT);
    mgl.enableDepthTest(true);

    // drawScene::run never touches the colour — it only passes alpha
    // through set_alpha — so the tint comes from the material, not the
    // script.
    // ogl::draw_display draws in two passes. An object is transparent if
    // any of its materials has opacity below 1 or is marked additive
    // (0x10005467); everything else goes in the opaque pass first, with
    // culling on and depth writes on. Then, at 0x10001ab6, blending goes
    // on, depth writes go off, culling goes off, and the transparent
    // objects follow — which is why the additive cubes read as six-sided:
    // nothing is culled and nothing occludes.
    // ogl_scene::disable_all_lights only enables GL_LIGHTING for a scene
    // that carries lights; four of the nine do.
    const lit = scene.lights && scene.lights.length > 0;
    mgl.enableLighting(lit);
    if (lit) mgl.setLights(this._sceneLights(scene, view, frame));

    const transparent = (mat) => !!mat && (mat.additive || mat.opacity < 1);
    const opaque = [], blended = [];
    for (const mesh of scene.meshes) {
      (transparent(scene.materials[mesh.src.material]) ? blended : opaque).push(mesh);
    }

    mgl.enableCullFace(true);
    mgl.cullFace(gl.BACK);
    mgl.depthMask(true);
    mgl.enableBlend(false);
    for (const mesh of opaque) this._drawMesh(scene, mesh, view, a, false);

    if (blended.length) {
      mgl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);
      mgl.enableBlend(true);
      mgl.depthMask(false);
      mgl.enableCullFace(false);
      for (const mesh of blended) this._drawMesh(scene, mesh, view, a, true);
    }

    mgl.enableCullFace(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    // ogl_scene::disable_all_lights runs on the way out too. Without this
    // the lighting stays on for the 2D layers that follow, which dims every
    // bitmap after a lit scene — bg.png in the dings section went grey.
    mgl.enableLighting(false);
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
    if (fx.constructor.kind === 2) this._setup3d();
    else this._setup2d();
    fx.draw(this.mgl, (time - inst.start) * 1000, p, this);
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
