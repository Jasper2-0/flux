// Bands (69 - 85s): twisting ribbon trails flying toward the camera, layered
// over the FFDEnv morph. Runs on absolute timeline time (fade-in at 69s,
// fade-out at 84s), like the original.

import { rand, RAND_MAX } from '../mathlib.js';

const BAND_SEGMENTS = 32;
const BAND_NEAR_PLANE = -1.0;
const BAND_FAR_PLANE = -50.0;

function frand() { return rand() / RAND_MAX; }

class Band {
  constructor(bandWidth, speedFactor) {
    this.bandWidth = bandWidth;
    this.speedFactor = speedFactor;
    // pts[s][side] = [x, y, z]
    this.pts = [];
    this.angles = new Float32Array(BAND_SEGMENTS + 1);
    this.draw = new Uint8Array(BAND_SEGMENTS + 1);
    for (let s = 0; s <= BAND_SEGMENTS; s++) {
      const z = BAND_NEAR_PLANE + (BAND_FAR_PLANE - BAND_NEAR_PLANE) * s / (BAND_SEGMENTS + 1);
      this.pts.push([[0, 0, z], [0, 0, z]]);
    }
  }

  render(mgl, alpha) {
    mgl.begin(mgl.QUADS);
    for (let s = 0; s < BAND_SEGMENTS; s++) {
      if (!this.draw[s]) continue;
      const a = alpha * (1.0 - s / BAND_SEGMENTS);
      mgl.color4(1, 1, 1, a);
      mgl.texCoord2(0, 0);
      mgl.vertex3(this.pts[s][0][0], this.pts[s][0][1], this.pts[s][0][2]);
      mgl.texCoord2(1, 0);
      mgl.vertex3(this.pts[s][1][0], this.pts[s][1][1], this.pts[s][1][2]);
      mgl.texCoord2(1, 1);
      mgl.vertex3(this.pts[s + 1][1][0], this.pts[s + 1][1][1], this.pts[s + 1][1][2]);
      mgl.texCoord2(0, 1);
      mgl.vertex3(this.pts[s + 1][0][0], this.pts[s + 1][0][1], this.pts[s + 1][0][2]);
    }
    mgl.end();
  }

  move(dz) {
    dz *= this.speedFactor;
    for (let s = 0; s < BAND_SEGMENTS; s++) {
      this.pts[s][0][2] += dz;
      this.pts[s][1][2] += dz;
    }
    while (this.pts[1][0][2] > BAND_NEAR_PLANE) {
      for (let s = 0; s < BAND_SEGMENTS; s++) {
        this.pts[s][0][0] = this.pts[s + 1][0][0];
        this.pts[s][0][1] = this.pts[s + 1][0][1];
        this.pts[s][0][2] = this.pts[s + 1][0][2];
        this.pts[s][1][0] = this.pts[s + 1][1][0];
        this.pts[s][1][1] = this.pts[s + 1][1][1];
        this.pts[s][1][2] = this.pts[s + 1][1][2];
        this.angles[s] = this.angles[s + 1];
        this.draw[s] = this.draw[s + 1];
      }
      const fx = frand() - 0.5;
      const fy = frand() - 0.5;
      const fa = (frand() - 0.5) * 2.0;
      const fr = 1.0 + 0.3 * frand();
      this.draw[BAND_SEGMENTS] = 1;
      let ang = this.angles[BAND_SEGMENTS - 1] + fa;
      ang = ang % (2.0 * 355.0 / 113.0);
      this.angles[BAND_SEGMENTS] = ang;
      this.pts[BAND_SEGMENTS][0][0] = fx + Math.sin(ang) * fr;
      this.pts[BAND_SEGMENTS][0][1] = fy + Math.cos(ang) * fr;
      this.pts[BAND_SEGMENTS][0][2] = BAND_FAR_PLANE;
      this.pts[BAND_SEGMENTS][1][0] = fx + Math.sin(ang + this.bandWidth) * fr;
      this.pts[BAND_SEGMENTS][1][1] = fy + Math.cos(ang + this.bandWidth) * fr;
      this.pts[BAND_SEGMENTS][1][2] = BAND_FAR_PLANE;
    }
  }
}

export class Bands {
  constructor(mgl, tex, nBands = 40) {
    this.mgl = mgl;
    this.bands = [];
    for (let i = 0; i < nBands; i++) {
      this.bands.push(new Band(0.1 + 0.2 * frand(), 1.0 + 1.0 * frand()));
    }
    this.tex = tex.loadTexture('data/textures/polka.png', true);
    this.lastTime = -1.0;
  }

  do(time /* absolute timeline time */) {
    const mgl = this.mgl;
    const delta = this.lastTime < 0 ? 0 : time - this.lastTime;
    this.lastTime = time;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-0.5, 0.5, -0.375, 0.375, -BAND_NEAR_PLANE, -BAND_FAR_PLANE);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();
    mgl.rotate(time * 50.0, 0, 0, 1);
    mgl.rotate(20.0 * Math.sin(time), 1, 0, 0);
    mgl.rotate(20.0 * Math.sin(time * 2.0), 0, 1, 0);

    mgl.enableCullFace(false);

    for (const band of this.bands) band.move(delta * 10.0);

    mgl.bindTexture(this.tex);
    mgl.enableTexture(true);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.enableDepthTest(false);

    let alpha = 1.0;
    if (time >= 69.0 && time < 70.0) alpha = time - 69.0;
    else if (time >= 84.0 && time < 85.0) alpha = 1.0 - (time - 84.0);

    for (const band of this.bands) band.render(mgl, alpha);
  }
}
