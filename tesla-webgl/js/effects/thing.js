// Thing / Rotator (222.5 - 254s): the credits outro — a swirl of large
// additive quads with drifting UVs beneath cross-fading credit overlays,
// opening with a white flash.

import { clamp01 } from '../mathlib.js';

const CHANGE_TIME = 8;
const CHANGE_TIME1 = 16;
const CHANGE_TIME3 = 26.5;

export class Thing {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.tex = tex.loadTexture('data/textures/y2.jpg');
    this.layer1 = tex.loadTexture('data/textures/cred-saffron01.png');
    this.layer2 = tex.loadTexture('data/textures/cred-yoghurt01.png');
    this.layer3 = tex.loadTexture('data/textures/cred-radixlluvia01.png');
  }

  _putQuad(u, v, scale = 1) {
    const mgl = this.mgl;
    mgl.begin(mgl.QUADS);
    mgl.texCoord2(u, v); mgl.vertex3(-300 * scale, 300 * scale, 0);
    mgl.texCoord2(1 + u, v); mgl.vertex3(300 * scale, 300 * scale, 0);
    mgl.texCoord2(1 + u, 1 + v); mgl.vertex3(300 * scale, -300 * scale, 0);
    mgl.texCoord2(u, 1 + v); mgl.vertex3(-300 * scale, -300 * scale, 0);
    mgl.end();
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = time - timeStart;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-0.6, 0.6, -0.45, 0.45, 1, 1000);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();
    mgl.translate(0, 80, -250);
    mgl.rotate(-60, 1, 0, 0);
    mgl.rotate(t * 10, 0, 0, 1);

    mgl.enableTexture(true);
    mgl.enableCullFace(false);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);

    mgl.bindTexture(this.tex);

    let alpha;
    if (t > CHANGE_TIME3) {
      alpha = 1 + CHANGE_TIME3 - t;
    } else {
      alpha = 1;
    }

    mgl.color4(1, 1, 1, (0.2 + 0.1 * Math.sin(t * 2)) * alpha);

    mgl.pushMatrix();
    for (let i = 0; i < 8; i++) {
      mgl.rotate(5 + Math.sin(t + i * 10) * 10, 0, 0, 1);
      const u = -(0.4 * Math.sin(t + i * 6) + 0.4 * Math.cos(t * 2 + i * 4)) * 0.1;
      const v = (0.3 * Math.sin(t + i * 12) + 0.4 * Math.sin(t + i * 5)) * 0.05;
      this._putQuad(u, v);
    }
    mgl.popMatrix();

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.ortho(-300, 300, -300, 300, -1, 1);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);

    if (t < CHANGE_TIME) {
      const fa = clamp01((CHANGE_TIME - t) * 0.3);

      mgl.bindTexture(this.layer1);
      mgl.color4(1, 1, 1, fa * alpha);
      this._putQuad(0, 0, 1);

      mgl.bindTexture(this.layer3);
      mgl.color4(1, 1, 1, (1 - fa) * alpha);
      this._putQuad(0, 0, 1);
    } else {
      const fa = clamp01((t - CHANGE_TIME1) * 0.3);

      mgl.bindTexture(this.layer2);
      mgl.color4(1, 1, 1, fa * alpha);
      this._putQuad(0, 0, 1);

      mgl.bindTexture(this.layer3);
      mgl.color4(1, 1, 1, (1 - fa) * alpha);
      this._putQuad(0, 0, 1);
    }

    // white flash carried over from the previous scene
    const bum = clamp01(1 - t * 0.5);
    if (bum > 0) {
      mgl.enableTexture(false);
      mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
      mgl.color4(1, 1, 1, bum);
      this._putQuad(0, 0, 1);
    }
  }
}
