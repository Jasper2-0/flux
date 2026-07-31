// Contour port runtime: plays the timeline extracted from contour.exe,
// dispatching create/kill records to effect implementations where they
// exist and to visible placeholders where they don't yet.

import { MiniGL } from './minigl.js';
import { parseARSE } from './arse.js';
import { Bloem } from './effects/bloem.js';
import { Rogplay } from './effects/rogplay.js';
import { TimScene } from './effects/timscene.js';
import { Zoomer } from './effects/zoomer.js';
import { Letters } from './effects/letters.js';
import { Credit, resetCreditOrder } from './effects/credits.js';
import { Grid1fx } from './effects/grid1fx.js';
import { Flarefx } from './effects/flarefx.js';
import { Flash } from './effects/flash.js';
import { Picflash } from './effects/picflash.js';
import { Linefx } from './effects/linefx.js';
import { IntroBurst, CreditsRough, ContLogoRough, EndCard } from './effects/quickparts.js';

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
  'data/textures/line6.jpg',
  'data/logo.jpg',
  'data/letters/abc.jpg',
  'data/credits/credits-tekst.jpg',
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
    this.font = JSON.parse(new TextDecoder().decode(await this._bytes('data/font-abc.json')));

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
    // (The zoomer is a real effect now — id 60, shown by its 0x20 at 108 s.)
    this.roughCut = [
      { name: '≈ intro burst', effect: new IntroBurst(this.mgl), window: [0, 12] },
      { name: '≈ credits backdrops', effect: new CreditsRough(this.mgl, this.tex), window: [12, 41.5] },
      { name: '≈ contour logo overlay', effect: new ContLogoRough(this.mgl, this.tex), window: [41.5, 52] },
      { name: '≈ end card', effect: new EndCard(this.mgl, this.tex), window: [174, 192.5] },
    ];

    this.status('ready');
  }

  // Effect factory for create records. The dispatcher semantics (decoded
  // from the exe): create = preload, message code 0x20 = show, kill =
  // destroy. Instances are keyed by id; implemented ids get an effect,
  // everything else shows as a placeholder.
  _makeEffect(record) {
    const refs = (record.params_refs || []).map((r) => r.str).join(' ') +
      ' ' + (record.params_str || '');

    // id 13 nixfx/contlogo runs Tim's replayer on dildo.bin (shown 40 s)
    if (refs.includes('dildo.bin')) {
      return { effect: new TimScene(this.mgl, this.tex, this.dildoScene, 'data/saftext/fx8.jpg') };
    }
    // `scenes\neuron.bin` was once dispatched here too, because the string
    // turned up in instance 455's parameter dump. Now that the flash block
    // is decoded as 32 bytes we know that pointer sits one dword *past*
    // the end of 455's block: it belongs to whatever comes next, not to a
    // flash. No timeline record references the neuron scene inside its own
    // block, so the port no longer renders it. The shipped scene stays in
    // data/ and the parser still validates against it.

    // Scid's poem: the parameter block carries text, drift and timing
    if (record.letters) {
      return { effect: new Letters(this.mgl, this.tex, this.font, record.letters) };
    }

    switch (record.instance) {
      case 'bloem':
        return { effect: new Bloem(this.mgl, this.tex) };
      case 'rogplay':
        return { effect: new Rogplay(this.mgl, this.tex) };
      case 'grid1fx':
        return { effect: new Grid1fx(this.mgl, this.tex) };
      case 'flarefx':
        return { effect: new Flarefx(this.mgl, this.tex) };
      case 'flash':
        return record.flash
          ? { effect: new Flash(this.mgl, this.tex, record.flash) }
          : null;
      case 'picflash':
        return record.picflash
          ? { effect: new Picflash(this.mgl, this.tex, record.picflash) }
          : null;
      case 'linefx':
        return record.linefx
          ? { effect: new Linefx(this.mgl, this.tex, record.linefx) }
          : null;
      case 'zoomer':
        return { effect: new Zoomer(this.mgl, this.tex, { ending: this.opts.zoomerEnding || 'original' }) };
      case 'credit3':
        return { effect: new Credit(this.mgl, this.tex) };
      default:
        return null;
    }
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
    resetCreditOrder();
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
      if (r.kind === 'create') {
        let made = null;
        try {
          made = this._makeEffect(r);
        } catch (e) { /* placeholder instead */ }
        // creates that are never messaged 0x20 (e.g. the timed credit3
        // rows) act on their cue directly, so they start shown
        const preload = r.time === 0;
        this.instances.set(r.id, {
          record: r,
          effect: made && made.effect,
          shown: !preload,
          shownAt: preload ? null : r.time,
        });
      } else if (r.kind === 'kill') {
        this.instances.delete(r.id);
      } else if (r.kind === 'message') {
        const inst = this.instances.get(r.id);
        if (inst) {
          if (r.code === '0x20' && !inst.shown) {
            inst.shown = true;
            inst.shownAt = r.time;
          }
          // other codes (0x9004/0x9005/0x9010/0xa001…) are per-effect
          // mode switches — recorded but not yet interpreted
          inst.lastCode = r.code;
        }
      }
      // musicctl / clockjump records don't affect the port's clock: it
      // follows the soundtrack element directly
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
      active.push({
        id,
        name: inst.record.class + '/' + inst.record.instance,
        implemented: !!inst.effect,
        rendering: !!inst.effect && inst.shown,
      });
      if (inst.effect && inst.shown) {
        // baked scenes stop when their frames run out (no explicit kill
        // record exists for them; the engine presumably self-hides)
        const dur = inst.effect.duration;
        if (!dur || time - (inst.shownAt || 0) <= dur) {
          try {
            // effect-local time runs from the moment it was shown
            inst.effect.do(time, inst.shownAt || 0);
          } catch (e) { /* keep the loop alive */ }
        }
      }
    }
    this.onActiveChange(time, active);
  }
}
