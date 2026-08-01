// Nowhere port runtime — plays the demo's own script.
//
// Axiom's script is a flat list of cue lines, `time instance param value`,
// where time is milliseconds and PREDEMO (-1000000) is the preload pass.
// REGISTER binds an instance name to an effect plugin; `execute load`
// loads whatever `filename` was set; `run true/false` shows and hides.
// Instances draw in `layer` order, high numbers first.
//
// The clock is the shipped soundtrack, exactly as the original's was
// (Axiom drove it from the MP3 byte position).

import { MiniGL } from './minigl.js';
import { ZeuScene, SCENE_FPS } from './zeus.js';
import { Backgr } from './effects/backgr.js';
import { Anim } from './effects/anim.js';
import { ZeusPlay } from './effects/zeusplay.js';
import { Sphere } from './effects/sphere.js';

const DEMO_LENGTH = 206.5;   // `sound length 206488`

// Effects that exist; anything else shows as a live placeholder so the
// structure of the demo is visible while the rest is being written.
const EFFECTS = {
  backgr: Backgr,
  anim: Anim,
  zeusplay: ZeusPlay,
  sphere: Sphere,
};

// Every other 3D part loads a .zeu and then deforms it procedurally —
// the sphere morphs between its two GeoSpheres, 33 turbulates a ball and
// plays a movie on four screens, smoke and slierten march cubes over
// blobs. Until each of those is written, the scene is played straight,
// which gets the camera, the timing and the geometry right and is
// labelled as such in the readout.
const RAW_SCENE = new Set(['credits', 'greets', 'fire', 'smoke', 'slierten', '33']);

class Assets {
  constructor(mgl, base = 'data/') {
    this.mgl = mgl;
    this.base = base;
    this.tex = new Map();
    this.scenes = new Map();
    this.images = new Map();
  }

  async image(path) {
    if (this.images.has(path)) return this.images.get(path);
    const img = new Image();
    img.src = this.base + path;
    const p = img.decode().then(() => img);
    this.images.set(path, p);
    return p;
  }

  // Textures are uploaded through a canvas: iOS Safari is unreliable
  // about ImageBitmap uploads, and this port has to run there.
  async texture(path, { alpha = null, mipmap = true } = {}) {
    const key = path + '|' + (alpha || '');
    if (this.tex.has(key)) return this.tex.get(key);
    const p = (async () => {
      const img = await this.image(path);
      const cv = document.createElement('canvas');
      cv.width = img.naturalWidth; cv.height = img.naturalHeight;
      const ctx = cv.getContext('2d', { willReadFrequently: true });
      ctx.drawImage(img, 0, 0);
      const id = ctx.getImageData(0, 0, cv.width, cv.height);
      if (alpha) {
        const am = await this.image(alpha);
        const ac = document.createElement('canvas');
        ac.width = am.naturalWidth; ac.height = am.naturalHeight;
        const actx = ac.getContext('2d', { willReadFrequently: true });
        actx.drawImage(am, 0, 0, cv.width, cv.height);
        const ad = actx.getImageData(0, 0, cv.width, cv.height).data;
        for (let i = 0; i < id.data.length; i += 4) id.data[i + 3] = ad[i];
      }
      return this.mgl.createTextureFromData(
        new Uint8Array(id.data.buffer), cv.width, cv.height, mipmap);
    })();
    this.tex.set(key, p);
    return p;
  }

  async scene(path) {
    if (this.scenes.has(path)) return this.scenes.get(path);
    const p = fetch(this.base + path.replace(/\.zeu$/i, '.json'))
      .then((r) => r.json())
      .then((j) => new ZeuScene(j));
    this.scenes.set(path, p);
    return p;
  }

  async textLines(path) {
    const t = await (await fetch(this.base + path)).text();
    return t.split(/\s+/).filter(Boolean).map(norm);
  }
}

const norm = (p) => String(p).replace(/\\/g, '/').replace(/^\.\//, '').toLowerCase();

class Instance {
  constructor(name, effectName, ctx) {
    this.name = name;
    this.effectName = effectName;
    this.params = new Map();
    this.running = false;
    this.startedAt = 0;
    this.layer = 0;
    this.ctx = ctx;
    const Cls = EFFECTS[effectName] || (RAW_SCENE.has(effectName) ? ZeusPlay : null);
    this.rawScene = !EFFECTS[effectName] && RAW_SCENE.has(effectName);
    this.effect = Cls ? new Cls(ctx, this) : null;
  }

  get(key, dflt) {
    const v = this.params.get(key);
    return v === undefined ? dflt : v;
  }

  num(key, dflt = 0) {
    const v = parseFloat(this.params.get(key));
    return Number.isFinite(v) ? v : dflt;
  }

  set(param, value, timeSec) {
    this.params.set(param, value);
    if (param === 'layer') this.layer = this.num('layer', 0);
    if (param === 'run') {
      const on = value === 'true' || value === '1';
      if (on && !this.running) this.startedAt = timeSec;
      this.running = on;
    }
    if (param === 'execute' && this.effect && this.effect.load) {
      this.loading = this.effect.load().catch((e) => { this.error = e.message; });
    }
  }
}

export class Demo {
  constructor(canvas, status = () => {}, opts = {}) {
    this.canvas = canvas;
    this.status = status;
    this.opts = opts;
    this.running = false;
    this.timeOverride = null;
    this.onActiveChange = opts.onActiveChange || (() => {});
  }

  async load() {
    this.status('initialising WebGL…');
    this.mgl = new MiniGL(this.canvas);
    this.assets = new Assets(this.mgl, this.opts.base || 'data/');

    this.status('loading the demo script…');
    this.script = await (await fetch((this.opts.base || 'data/') + 'timeline.json')).json();

    this.ctx = {
      mgl: this.mgl,
      assets: this.assets,
      materials: this.script.materials,
      textures: this.script.textures,
      fps: SCENE_FPS,
    };

    this.instances = new Map();
    this.cursor = 0;

    this.status('preloading…');
    await this._runCues(-1);           // the PREDEMO pass
    for (const inst of this.instances.values()) {
      if (inst.loading) await inst.loading;
    }

    this.status('loading the soundtrack…');
    this.audio = new Audio((this.opts.base || 'data/') + 'sound.mp3');
    this.audio.preload = 'auto';
    this.status('ready');
  }

  _instance(name) {
    let inst = this.instances.get(name);
    if (!inst) {
      const eff = this.script.registrations[name] || name;
      inst = new Instance(name, eff, this.ctx);
      this.instances.set(name, inst);
    }
    return inst;
  }

  async _runCues(uptoSec) {
    const ms = uptoSec * 1000;
    while (this.cursor < this.script.timeline.length &&
           this.script.timeline[this.cursor].ms <= ms) {
      const cue = this.script.timeline[this.cursor++];
      if (cue.instance === 'system') continue;
      this._instance(cue.instance).set(cue.param, cue.value, cue.ms / 1000);
    }
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

  getTime() {
    if (this.timeOverride !== null) return this.timeOverride;
    if (this.audio && !this.audio.paused && this.audio.currentTime > 0) return this.audio.currentTime;
    return performance.now() / 1000 - this.startClock;
  }

  seek(t) {
    t = Math.max(0, Math.min(DEMO_LENGTH, t));
    if (this.audio && !this.audio.paused) this.audio.currentTime = t;
    this.startClock = performance.now() / 1000 - t;
    // replay the script from the top so instance state is exact
    this.cursor = 0;
    for (const inst of this.instances.values()) {
      inst.params.clear(); inst.running = false;
    }
    this._runCues(t);
  }

  renderFrame(time) {
    this._runCues(time);
    this.mgl.clear();

    const active = [];
    const live = [...this.instances.values()].filter((i) => i.running);
    // The plugins' help text says "high numbers are drawn before low
    // numbers", but the script only makes sense the other way round:
    // backgrounds sit at layer 0, the 3D parts at 1, the sprite overlays
    // at 2. Ascending it is — higher layers land in front.
    live.sort((a, b) => a.layer - b.layer);
    for (const inst of live) {
      const ready = !!(inst.effect && inst.effect.ready);
      active.push({ name: inst.name, effect: inst.effectName, layer: inst.layer, ready, raw: inst.rawScene });
      if (ready) {
        try {
          inst.effect.draw(time - inst.startedAt, time);
        } catch (e) {
          inst.error = e.message;
        }
      }
    }
    this.onActiveChange(time, active);
  }
}
