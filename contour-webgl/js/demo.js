// Contour port runtime: plays the timeline extracted from contour.exe,
// dispatching create/kill records to effect implementations where they
// exist and to visible placeholders where they don't yet.

import { MiniGL } from './minigl.js';
import { parseARSE } from './arse.js';
import { Bloem } from './effects/bloem.js';
import { Rogplay } from './effects/rogplay.js';
import { TimScene } from './effects/timscene.js';
import { IntroBurst, CreditsRough, ContLogoRough, ZoomerLite, EndCard } from './effects/quickparts.js';

const DEMO_LENGTH = 194; // shipped soundtrack runs 3:14

// Async texture manager over loose files (Contour's assets are extracted
// to data/, unlike Tesla's pak). An `embed` map (path -> {b64, mime})
// substitutes data URIs for fetches in single-file builds.
class TexManager {
  constructor(mgl, embed = null) {
    this.mgl = mgl;
    this.embed = embed;
    this.cache = new Map();
  }
  async preload(path) {
    if (this.cache.has(path)) return this.cache.get(path);
    const img = new Image();
    const e = this.embed && this.embed[path];
    img.src = e ? 'data:' + e.mime + ';base64,' + e.b64 : path;
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
  'data/saftext/fx4.jpg',
  'data/saftext/fx8.jpg',
  'data/saftext/fx9.jpg',
  'data/textures/flare1.jpg',
  'data/textures/flare8.jpg',
  'data/textures/dildo.jpg',
  'data/logo.jpg',
  'data/contourlogo-overlay-image01.jpg',
  'data/credits/credbk-1-image.jpg',
  'data/credits/credbk-2-image.jpg',
  'data/credits/credbk-3-image.jpg',
  'data/credits/credbk-4-image.jpg',
  'data/credits/credbk-5-image.jpg',
  'data/zoomer/image0a.jpg', 'data/zoomer/image1a.jpg', 'data/zoomer/image2a.jpg',
  'data/zoomer/image3a.jpg', 'data/zoomer/image4a.jpg', 'data/zoomer/image5a.jpg',
  'data/zoomer/image6a.jpg', 'data/zoomer/image7a.jpg', 'data/zoomer/image8a.jpg',
  'data/zoomer/image9a.jpg', 'data/zoomer/shun-compo.jpg',
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

  async _bytes(path) {
    const e = this.opts.embed && this.opts.embed[path];
    if (e) {
      const bin = atob(e.b64);
      const out = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
      return out;
    }
    return new Uint8Array(await (await fetch(path)).arrayBuffer());
  }

  async load() {
    this.status('initializing WebGL…');
    this.mgl = new MiniGL(this.canvas);
    this.tex = new TexManager(this.mgl, this.opts.embed || null);

    this.status('loading timeline…');
    const tl = JSON.parse(new TextDecoder().decode(await this._bytes('data/timeline.json')));
    this.timeline = tl.records.filter((r) => r.class);

    this.status('loading textures…');
    for (const t of TEXTURES) await this.tex.preload(t);

    this.status('loading scenes…');
    this.dildoScene = parseARSE(await this._bytes('data/scenes/dildo.bin'));
    this.neuronScene = parseARSE(await this._bytes('data/scenes/neuron.bin'));

    this.status('loading soundtrack…');
    const embAudio = this.opts.embed && this.opts.embed['data/test.mp3'];
    this.audio = new Audio(embAudio
      ? 'data:audio/mpeg;base64,' + embAudio.b64
      : 'data/test.mp3');
    this.audio.preload = 'auto';

    // Rough-cut fillers keep the show continuous where the real parts are
    // not reverse-engineered yet. Windows observed from the release AVI.
    this.roughCut = [
      { name: '≈ intro burst', effect: new IntroBurst(this.mgl), window: [0, 12] },
      { name: '≈ credits backdrops', effect: new CreditsRough(this.mgl, this.tex), window: [12, 41.5] },
      { name: '≈ contour logo', effect: new ContLogoRough(this.mgl, this.tex), window: [41.5, 52] },
      { name: '≈ zoomer (lite)', effect: new ZoomerLite(this.mgl, this.tex), window: [108.5, 174] },
      { name: '≈ end card', effect: new EndCard(this.mgl, this.tex), window: [174, 192.5] },
    ];

    this.status('ready');
  }

  // Effect factory: instance-name keyed. Everything unimplemented returns
  // null and shows up in the placeholder readout instead.
  //
  // Timing caveat: most creates fire at t=0 as preloads; visibility is
  // driven by kind-2/3 messages that are not decoded yet. Until they are,
  // implemented effects carry an AVI-calibrated visible window
  // [start, end] observed from the release capture.
  _makeEffect(record) {
    switch (record.instance) {
      case 'bloem':
        return { effect: new Bloem(this.mgl, this.tex), window: [88, 108.5] };
      case 'rogplay':
        return { effect: new Rogplay(this.mgl, this.tex), window: [64, 82] };
      default:
        break;
    }
    // scene files ride in parameter blocks: dildo.bin on the static
    // credit5 record, neuron.bin on the linefx id=455 preload
    const refs = (record.params_refs || []).map((r) => r.str).join(' ') +
      ' ' + (record.params_str || '');
    if (refs.includes('neuron.bin')) {
      // texture guess: the capture shows coppery surfaces — fx8/dildo.jpg
      return {
        effect: new TimScene(this.mgl, this.tex, this.neuronScene, 'data/saftext/fx8.jpg'),
        window: [52, 65.5],
      };
    }
    if (refs.includes('dildo.bin')) {
      // not yet located in the capture; parked until the messages are
      // decoded (the scene is 17 s long — plausibly the intro)
      return null;
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
        let made = null;
        try {
          made = this._makeEffect(r);
        } catch (e) { /* placeholder instead */ }
        this.instances.set(r.id, {
          record: r,
          effect: made && made.effect,
          window: made && made.window,
          born: r.time,
        });
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

    // rough-cut fillers render first, underneath any real effects
    for (const rc of this.roughCut) {
      if (time >= rc.window[0] && time <= rc.window[1]) {
        active.push({ id: '~', name: rc.name + ' (rough cut)', implemented: true, rendering: true });
        try {
          rc.effect.do(time, rc.window[0]);
        } catch (e) { /* keep the loop alive */ }
      }
    }

    for (const [id, inst] of this.instances) {
      const inWindow = !inst.window || (time >= inst.window[0] && time <= inst.window[1]);
      active.push({
        id,
        name: inst.record.class + '/' + inst.record.instance,
        implemented: !!inst.effect,
        rendering: !!inst.effect && inWindow,
      });
      if (inst.effect && inWindow) {
        try {
          // effect-local time runs from its window start (AVI-calibrated)
          inst.effect.do(time, inst.window ? inst.window[0] : inst.born);
        } catch (e) { /* keep the loop alive */ }
      }
    }
    this.onActiveChange(time, active);
  }
}
