// Tesla demo runtime: loads data.pak, builds all effects, and plays the
// 254-second timeline synchronized to the soundtrack.

import { MiniGL } from './minigl.js';
import { Pak, TexManager } from './pak.js';
import { parse3DS } from './t3ds.js';

import { SpinZoom } from './effects/spinzoom.js';
import { ShadeBall } from './effects/shadeball.js';
import { Splines } from './effects/splines.js';
import { FFDEnv } from './effects/ffdenv.js';
import { Bands } from './effects/bands.js';
import { EnergyStream } from './effects/energystream.js';
import { Tubes } from './effects/tubes.js';
import { PolkaLike } from './effects/polkalike.js';
import { Tree } from './effects/tree.js';
import { FaceMorph } from './effects/facemorph.js';
import { Thing } from './effects/thing.js';

const DEMO_LENGTH = 254;

const TEXTURES = [
  ['data/textures/sphere.jpg'],
  ['data/textures/gothickiemura02.jpg'],
  ['data/textures/kalatus1-01.png'],
  ['data/textures/flare02.jpg'],
  ['data/textures/max_t3.jpg'],
  ['data/textures/max_t1.jpg'],
  ['data/textures/blend.tga'],
  ['data/textures/blend2.png'],
  ['data/textures/polka.png'],
  ['data/textures/polka.png', true],
  ['data/textures/y2.jpg'],
  ['data/textures/y6.jpg'],
  ['data/textures/y7.jpg'],
  ['data/textures/t1a.jpg'],
  ['data/textures/t1b.jpg'],
  ['data/textures/face.jpg'],
  ['data/textures/cred-saffron01.png'],
  ['data/textures/cred-yoghurt01.png'],
  ['data/textures/cred-radixlluvia01.png'],
];

export class Demo {
  constructor(canvas, statusCallback = () => {}) {
    this.canvas = canvas;
    this.status = statusCallback;
    this.effects = [];
    this.audio = null;
    this.startClock = 0;
    this.running = false;
    this.timeOverride = null; // for testing: fixed timeline position
  }

  async load() {
    this.status('initializing WebGL...');
    this.mgl = new MiniGL(this.canvas);

    this.status('downloading data.pak...');
    const pakResp = await fetch('data/data.pak');
    if (!pakResp.ok) throw new Error('failed to fetch data.pak');
    this.pak = new Pak(await pakResp.arrayBuffer());

    this.status('decoding textures...');
    this.tex = new TexManager(this.mgl, this.pak);
    for (const [name, mip] of TEXTURES) {
      await this.tex.preload(name, mip);
    }

    this.status('parsing meshes...');
    const metaScene = parse3DS(this.pak.get('data/3d/meta.3ds'));
    const facesScene = parse3DS(this.pak.get('data/3d/faces.3ds'));
    const kbuuScene = parse3DS(this.pak.get('data/3d/kbuu2.3ds'));

    this.status('building effects...');
    const mgl = this.mgl, tex = this.tex;
    // (effect, timeStart, timeEnd) — the original OnCreate() timeline
    this.effects = [
      [new SpinZoom(mgl, tex), 0, 24.5],
      [new ShadeBall(mgl, tex), 24.5, 48.5],
      [new Splines(mgl, tex), 48.5, 67.7],
      [new FFDEnv(mgl, tex, metaScene), 67.7, 87],
      [new Bands(mgl, tex), 69, 85],
      [new EnergyStream(mgl, tex), 87, 145],
      [new Tubes(mgl, tex), 145, 165],
      [new PolkaLike(mgl, tex), 165, 203],
      [new Tree(mgl, tex, kbuuScene), 165 + 21, 201],
      [new FaceMorph(mgl, tex, facesScene), 203, 222.5],
      [new Thing(mgl, tex), 222.5, 254],
    ];

    this.status('loading soundtrack...');
    this.audio = new Audio('data/tournesol.mp3');
    this.audio.preload = 'auto';

    this.status('ready');
  }

  start() {
    if (this.running) return;
    this.running = true;
    if (this.audio) {
      this.audio.currentTime = 0;
      this.audio.play().catch(() => { /* keep running without sound */ });
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
    if (this.audio && !this.audio.paused) {
      this.audio.currentTime = t;
    }
    this.startClock = performance.now() / 1000 - t;
  }

  getTime() {
    if (this.timeOverride !== null) return this.timeOverride;
    // follow the soundtrack when it is playing, else the wall clock
    if (this.audio && !this.audio.paused && this.audio.currentTime > 0) {
      return this.audio.currentTime;
    }
    return performance.now() / 1000 - this.startClock;
  }

  renderFrame(time) {
    const mgl = this.mgl;
    mgl.clear();

    if (time >= DEMO_LENGTH) {
      this.stop();
      return;
    }

    for (const [effect, tStart, tEnd] of this.effects) {
      if (time > tStart && time < tEnd) {
        effect.do(time, tStart);
      }
    }
  }
}
