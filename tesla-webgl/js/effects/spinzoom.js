// SpinZoom (0 - 24.5s): the intro — a stack of 18 spinning additive quads
// with a keyframed brightness envelope and dark scanline overlays.

import { darkQuads } from './common.js';

const Z_MAX = 300;

// (time, brightness) envelope from the original
const ENV = [
  [0, 0], [3.3, 0],
  [6.5, 1], [8, 1],
  [9, 0], [10, 0],
  [13, 1], [15, 0],
  [16, 1],
  [20, 0], [22, 1],
  [24, 0], [99, 0], [99, 0], [99, 0], [99, 0], [99, 0],
];

export class SpinZoom {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.tex = tex.loadTexture('data/textures/sphere.jpg');
    this.scaleChange = 1.02;
  }

  _putQuad() {
    const mgl = this.mgl;
    mgl.begin(mgl.QUADS);
    mgl.texCoord2(0, 0); mgl.vertex3(-1, 1, 0);
    mgl.texCoord2(1, 0); mgl.vertex3(1, 1, 0);
    mgl.texCoord2(1, 1); mgl.vertex3(1, -1, 0);
    mgl.texCoord2(0, 1); mgl.vertex3(-1, -1, 0);
    mgl.end();
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = time - timeStart - 0.1;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-0.6, 0.6, -0.45, 0.45, 1, Z_MAX);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableTexture(true);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.enableCullFace(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);

    mgl.bindTexture(this.tex);

    // envelope segment lookup
    let u = 0;
    for (u = 0; u < ENV.length - 1; u++) {
      if (t > ENV[u][0] && t < ENV[u + 1][0]) break;
    }
    const span = ENV[u + 1][0] - ENV[u][0];
    const bTime = (t - ENV[u][0]) / span;
    const unrMult = ENV[u][1] + (ENV[u + 1][1] - ENV[u][1]) * bTime;

    mgl.color4(1, 1, 1, unrMult * 0.23);

    mgl.pushMatrix();
    mgl.translate(0, 0, -2);
    mgl.rotate(t, 0, 1, 0);
    mgl.rotate(t * 0.5123, 1, 0, 0);

    for (let i = 0; i < 18; i++) {
      switch (u) {
        case 5:
        case 6: {
          mgl.rotate(t * 3 + 5, 1, -0.2, -0.3);
          mgl.color4(1, 1, 1, unrMult * 0.33);
          const uM = Math.sin(bTime * Math.PI / 2);
          if (ENV[u + 1][1] >= ENV[u][1]) {
            mgl.scale(this.scaleChange * uM, this.scaleChange, 1);
          } else {
            mgl.scale(this.scaleChange, this.scaleChange, 1);
          }
          break;
        }
        case 7:
        case 8:
          mgl.rotate(t * 22 + 5, 1, 0, 0);
          mgl.rotate(t * 2 + 5, 0, 0, 1);
          mgl.color4(1, 1, 1, unrMult * 0.33);
          break;
        case 9:
        case 10:
          mgl.rotate(10 * Math.sin(t) + 5, 1, -0.2, 0.3);
          mgl.rotate(5 * Math.cos(t) + 10 * Math.cos(t / 2), 0, 0.5, 1);
          if (ENV[u + 1][1] < ENV[u][1]) {
            mgl.scale(this.scaleChange * unrMult, this.scaleChange, 1);
          } else {
            mgl.scale(this.scaleChange, this.scaleChange, 1);
          }
          break;
        default:
          mgl.rotate(10 * Math.sin(t * 0.9) + 5, 1, -0.2, 0.3);
          if (ENV[u + 1][1] > ENV[u][1]) {
            mgl.scale(this.scaleChange * unrMult * unrMult, this.scaleChange, 1);
          } else {
            mgl.scale(this.scaleChange, this.scaleChange, 1);
          }
          break;
      }
      this._putQuad();
    }
    mgl.popMatrix();

    switch (u) {
      case 5:
      case 6:
        darkQuads(mgl, 20, [0, 0, 0.1, 0.12]);
        break;
      case 7:
      case 8:
        darkQuads(mgl, 220, [0, 0, 0.041, 0.1]);
        break;
      default:
        darkQuads(mgl, 80, [0, 0, 0, 0.2]);
    }
  }
}
