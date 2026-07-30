// Contour port runtime: plays the timeline extracted from contour.exe,
// dispatching create/kill records to effect implementations where they
// exist and to visible placeholders where they don't yet.

import { MiniGL } from './minigl.js';
import { parseARSE } from './arse.js';
import { Bloem } from './effects/bloem.js';
import { Rogplay } from './effects/rogplay.js';
import { TimScene } from './effects/timscene.js';

const DEMO_LENGTH = 194; // shipped soundtrack runs 3:14

// Async texture manager over loose files (Contour's assets are extracted
// to data/, unlike Tesla's pak).
class TexManager {
  constructor(mgl) {
    this.mgl = mgl;
    this.cache = new Map();
  }
  async preload(path) {
    if (this.cache.has(path)) return this.cache.get(path);
    const img = new Image();
    img.src = path;
    await img.decode();
    const cv = document.createElement('canvas');
    cv.width = img.naturalWidth;
    cv.height = img.naturalHeight;
    const ctx = cv.getContext('2d', { willReadFrequently: true });
    ctx.drawImage(img, 0, 0);
    const id = ctx.getImageData(0, 0, cv.width, cv.height);
    const tex = this.mgl.createTextureFromData(new Uint8Array(id.data.buffer), cv.width, cv.height);
    this.cache.set(path, tex);
    return tex;
  }
  loadTexture(path) {
    const tex = this.cache.get(path);
    if (!tex) throw new Error('texture not preloaded: ' + path);
    return tex;
  }
}

const TEXTURES = [
  'data/saftext/fx8.jpg',
  'data/textures/flare1.jpg',
  'data/textures/flare8.jpg',
  'data/textures/dildo.jpg',
  'data/logo.jpg',
];

export class Demo {
  constructor(canvas, statusCallback = () => {}, opts = {}) {
    this.canvas = canvas;
    this.status = statusCallback;
    this.opts = opts;
    this.running = false;
    this.startClock = 0;
    this.timeOverride = null;
    this.instances = new Map(); // id -> {record, effect|null}
    this.processed = new Set();
    this.onActiveChange = opts.onActiveChange || (() => {});
  }

  async load() {
    this.status('initializing WebGL…');
    this.mgl = new MiniGL(this.canvas);
    this.tex = new TexManager(this.mgl);

    this.status('loading timeline…');
    const tl = await (await fetch('data/timeline.json')).json();
    this.timeline = tl.records.filter((r) => r.class);

    this.status('loading textures…');
    for (const t of TEXTURES) await this.tex.preload(t);

    this.status('loading scenes…');
    const dildoBytes = new Uint8Array(await (await fetch('data/scenes/dildo.bin')).arrayBuffer());
    this.dildoScene = parseARSE(dildoBytes);
    const neuronBytes = new Uint8Array(await (await fetch('data/scenes/neuron.bin')).arrayBuffer());
    this.neuronScene = parseARSE(neuronBytes);

    this.status('loading soundtrack…');
    this.audio = new Audio('data/test.mp3');
    this.audio.preload = 'auto';

    this.status('ready');
  }

  // Effect factory: instance-name keyed. Everything unimplemented returns
  // null and shows up in the placeholder readout instead.
  _makeEffect(record) {
    const key = record.class + '/' + record.instance;
    switch (record.instance) {
      case 'bloem':
        return new Bloem(this.mgl, this.tex);
      case 'rogplay':
        return new Rogplay(this.mgl, this.tex);
      default:
        break;
    }
    // TimScene rides on records whose params point at a scene file
    if (record.params_str && record.params_str.includes('dildo.bin')) {
      return new TimScene(this.mgl, this.tex, this.dildoScene, 'data/textures/dildo.jpg');
    }
    return null;
  }

  start() {
    if (this.running) return;
    this.running = true;
    if (this.audio) {
      this.audio.currentTime = 0;
      this.audio.play().catch(() => {});
    }
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
    t = Math.max(0, Math.min(DEMO_LENGTH, t));
    if (this.audio && !this.audio.paused) this.audio.currentTime = t;
    this.startClock = performance.now() / 1000 - t;
    // rebuild instance state from scratch for a clean seek
    this.instances.clear();
    this.processed.clear();
  }

  getTime() {
    if (this.timeOverride !== null) return this.timeOverride;
    if (this.audio && !this.audio.paused && this.audio.currentTime > 0) {
      return this.audio.currentTime;
    }
    return performance.now() / 1000 - this.startClock;
  }

  _dispatch(time) {
    for (const r of this.timeline) {
      if (r.time > time) continue;
      const stamp = r.va;
      if (this.processed.has(stamp)) continue;
      this.processed.add(stamp);
      if (r.kind === 0) {
        let effect = null;
        try {
          effect = this._makeEffect(r);
        } catch (e) { /* placeholder instead */ }
        this.instances.set(r.id, { record: r, effect, born: r.time });
      } else if (r.kind === 1) {
        this.instances.delete(r.id);
      }
      // kinds 2-4 are messages; parameters not decoded yet
    }
  }

  renderFrame(time) {
    const mgl = this.mgl;
    this._dispatch(time);
    mgl.clear();

    const active = [];
    for (const [id, inst] of this.instances) {
      active.push({
        id,
        name: inst.record.class + '/' + inst.record.instance,
        implemented: !!inst.effect,
      });
      if (inst.effect) {
        try {
          inst.effect.do(time, inst.born);
        } catch (e) { /* keep the loop alive */ }
      }
    }
    this.onActiveChange(time, active);
  }
}
