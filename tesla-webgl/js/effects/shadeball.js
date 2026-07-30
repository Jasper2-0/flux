// ShadeBall (24.5 - 48.5s): 20 nested, time-offset textured spheres rendered
// additively (front-face culled) with a keyframed alpha, plus an overlay.

import { makeSphere, keyVal } from './common.js';
import { clamp01 } from '../mathlib.js';

const KEYS = [
  [0, 0], [1.5, 1], [5, 1], [7, 0], [9, 1],
  [14.5, 1], [16, 0], [18.0, 1], [20, 1], [24, 0],
];

export class ShadeBall {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.tex = tex.loadTexture('data/textures/gothickiemura02.jpg');
    this.texLayer1 = tex.loadTexture('data/textures/kalatus1-01.png');
    this.sphere = makeSphere(20, 13, 13);
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t0 = time - timeStart;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-0.6, 0.6, -0.45, 0.45, 1, 1000);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableTexture(true);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.enableCullFace(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);

    mgl.translate(0, 0, -500);

    const alphaKey = clamp01(keyVal(KEYS, t0));

    mgl.bindTexture(this.tex);

    mgl.enableCullFace(true);
    mgl.cullFace(mgl.FRONT);

    let t = t0;
    for (let i = 0; i < 20; i++) {
      const alpha = 1 - i / 12;

      mgl.pushMatrix();
      mgl.rotate(t * 40 + 20 * Math.sin(t), 0, 1, 0);
      mgl.rotate(t * 60 + 30 * Math.sin(t * 1.2) + 20 * Math.sin(t * 2.1), 0, 0, 1);
      mgl.rotate(t * 50, 1, 0, 0);
      mgl.scale(alpha * 20 + 0.5, alpha * 20 + 0.5, alpha * 20 + 0.5);

      mgl.color4(1, 1, 1, 0.3 * alpha * alphaKey);
      mgl.drawElements(this.sphere.vertices, this.sphere.uv, this.sphere.faces);

      mgl.popMatrix();

      t -= 0.014 * (8 * Math.sin(t0) + 12);
    }

    mgl.cullFace(mgl.BACK);
    mgl.enableCullFace(false);

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.ortho(-300, 300, -300, 300, -1, 1);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);

    mgl.bindTexture(this.texLayer1);
    mgl.color4(1, 1, 1, alphaKey);
    mgl.begin(mgl.QUADS);
    mgl.texCoord2(0, 0); mgl.vertex3(-300, 300, 0);
    mgl.texCoord2(1, 0); mgl.vertex3(300, 300, 0);
    mgl.texCoord2(1, 1); mgl.vertex3(300, -300, 0);
    mgl.texCoord2(0, 1); mgl.vertex3(-300, -300, 0);
    mgl.end();
  }
}
