// Splines (48.5 - 67.7s): a rotating cage of scaled additive spheres plus 16
// Lissajous flare trails billboarded against the camera.

import { makeSphere } from './common.js';
import { clamp01, Vec3 } from '../mathlib.js';

class SplineTrail {
  constructor(tex, tailLen, xSin, ySin, zSin) {
    this.tex = tex;
    this.tailLen = tailLen;
    this.xSin = xSin;
    this.ySin = ySin;
    this.zSin = zSin;
  }

  render(mgl, time) {
    const cam = mgl.getModelView();
    cam.inverse();

    const cX = cam.baseX().mulSelf(10);
    const cY = cam.baseY().mulSelf(10);
    const pos = new Vec3();

    mgl.blendFunc(mgl.SRC_COLOR, mgl.ONE);
    mgl.bindTexture(this.tex);

    mgl.begin(mgl.QUADS);

    let t = time;
    for (let i = 0; i < this.tailLen; i++) {
      pos.x = Math.sin(t * this.xSin.x) * this.xSin.z + Math.sin(t * this.xSin.y + 1) * this.xSin.z;
      pos.y = Math.sin(t * this.ySin.x) * this.ySin.z + Math.sin(t * this.ySin.y + 1) * this.ySin.z;
      pos.z = Math.sin(t * this.zSin.x) * this.zSin.z + Math.sin(t * this.zSin.y + 1) * this.zSin.z;

      let alpha = clamp01(1 - i / this.tailLen);
      alpha *= alpha; alpha *= alpha;

      let k = Math.sin(alpha * 3.14 + 5);

      mgl.color4(1, 1, 0.9, 0.1 * alpha);

      if (t > 3) {
        const q = clamp01(t - 3);
        k = 1 - q + q * k;
      } else {
        k = 1;
      }

      mgl.texCoord2(0, 0);
      mgl.vertex3v(pos.sub(cX).addSelf(cY).mulSelf(k));
      mgl.texCoord2(1, 0);
      mgl.vertex3v(pos.add(cX).addSelf(cY).mulSelf(k));
      mgl.texCoord2(1, 1);
      mgl.vertex3v(pos.add(cX).sub(cY).mulSelf(k));
      mgl.texCoord2(0, 1);
      mgl.vertex3v(pos.sub(cX).sub(cY));

      t -= 0.036;
    }

    mgl.end();
  }
}

export class Splines {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.flareTex = tex.loadTexture('data/textures/flare02.jpg');
    this.tex = tex.loadTexture('data/textures/sphere.jpg');
    this.sphere = makeSphere(600, 10, 10);

    const V = (x, y, z) => new Vec3(x, y, z);
    const params = [
      [V(1.2, 2.3, 100), V(2.3, 2.13, 100), V(1.32, 2.16, 100)],
      [V(2.122, 1.3, 80), V(3.3, 1.13, 100), V(2.43, 1.26, 100)],
      [V(1.32, 1.14, 100), V(2.35, 2.223, 100), V(1.132, 2.416, 100)],
      [V(1.52, 2.23, 50), V(1.63, 2.613, 100), V(1.42, 3.06, 100)],
      [V(2.256, 2.23, 100), V(1.03, 1.213, 100), V(1.532, 2.216, 70)],
      [V(2.2, 1.3, 50), V(1.73, 2.413, 120), V(1.332, 2.916, 100)],
      [V(4.2, 1.3, 100), V(1.3, 1.13, 100), V(2.32, 2.16, 150)],
      [V(1.22, 0.93, 80), V(0.9, 1.913, 80), V(1.32, 2.16, 110)],
      [V(1.12, 2.3, 140), V(2.3, -2.13, 130), V(1.32, -2.16, 150)],
      [V(-2.212, 1.3, 160), V(3.3, 1.13, -150), V(-2.43, 1.26, 140)],
      [V(1.23, -1.14, 200), V(2.35, -2.223, 150), V(1.132, 2.416, -120)],
      [V(1.25, 2.23, 100), V(1.63, -2.613, 100), V(-1.42, -3.06, 160)],
      [V(-2.25, 2.23, 200), V(1.03, 1.213, -150), V(1.532, 2.216, 140)],
      [V(2.2, -1.3, 100), V(-1.73, 2.413, 190), V(-1.332, 2.916, 200)],
      [V(3.62, 2.3, 150), V(1.3, -1.13, -120), V(2.32, -2.16, 150)],
      [V(1.22, -0.93, 160), V(0.9, -1.913, 120), V(-1.32, 2.16, 210)],
    ];
    this.splines = params.map(([x, y, z]) => new SplineTrail(this.flareTex, 64, x, y, z));
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = time - timeStart;
    const aspect = 480 / 640;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-3.1, 3.1, -3.1 * aspect, 3.1 * aspect, 1.0, 2300.0);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.depthFunc(mgl.LEQUAL);

    mgl.enableCullFace(false);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);

    mgl.enableTexture(true);

    mgl.translate(0, 0, -500);

    mgl.bindTexture(this.tex);

    mgl.rotate(t * 15, 0, 1, 0);
    mgl.rotate(90 * Math.sin(t / 2) + 90 * Math.sin(t * 0.9), 0, 0, 1);

    mgl.pushMatrix();

    mgl.color4(1, 1, 1, 0.3 + 0.1 * Math.sin(t * 2));

    for (let i = 0; i < 8; i++) {
      mgl.rotate(6 * t * i, 0, 1, 0);
      mgl.rotate(5, 0, 0, 1);
      mgl.scale(1.1, 1.1, 1.1);
      mgl.drawElements(this.sphere.vertices, this.sphere.uv, this.sphere.faces);
    }

    mgl.popMatrix();

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-0.5, 0.5, -0.5 * aspect, 0.5 * aspect, 1.0, 2300.0);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    let tr = 0;
    if (t > 3) {
      tr = clamp01((t - 3) * 0.3) * 180;
    }
    mgl.translate(tr, 0, -500);
    mgl.rotate(t * 15, 0, 1, 0);
    mgl.rotate(90 * Math.sin(t / 2) + 90 * Math.sin(t * 0.9), 0, 0, 1);

    for (const spline of this.splines) {
      spline.render(mgl, t);
    }
  }
}
