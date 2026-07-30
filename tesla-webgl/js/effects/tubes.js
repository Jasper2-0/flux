// Tubes (145 - 165s): pulsating spline-tessellated tube (CSplinedObject with a
// wrapped 6x8 control grid) surrounded by 8 static textured cylinders, with a
// small face quad in the middle.

import { makeTube } from './common.js';
import { SplinedObject, ObjectNode } from './tessellator.js';
import { SinPulse } from '../mathlib.js';
import { clamp01 } from '../mathlib.js';

const TUBE_U = 6;
const TUBE_V = 8;

const CHANGE_TIME1 = 20 / 3;
const CHANGE_TIME2 = 40 / 3;
const CHANGE_TIME3 = 19.0;

export class Tubes {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.tex1 = tex.loadTexture('data/textures/t1b.jpg');
    this.tex2 = tex.loadTexture('data/textures/face.jpg');

    this.sobject = new SplinedObject();
    this.sobject.setMultiTexData([
      { tex: this.tex1, alpha: 1.0, uShift: 0, vShift: 0, material: 1 },
      { tex: this.tex1, alpha: 1.0, uShift: 0, vShift: 0, material: 2 },
    ]);
    this.sobject.setSeed(4, 6);

    this.table = [];
    for (let i = 0; i < TUBE_U * TUBE_V; i++) this.table.push(new ObjectNode());
    this.sobject.addTable(this.table, TUBE_U, TUBE_V);
    this.sobject.closeTableU();

    this.tube = makeTube(90, 8, 8);
    this.time = 0;
  }

  _renderTube(scale, angle0) {
    const mgl = this.mgl;
    mgl.enableTexture(true);
    mgl.bindTexture(this.tex1);

    mgl.pushMatrix();
    mgl.rotate(angle0, 0, 1, 0);
    mgl.scale(scale, scale, scale);
    mgl.drawElements(this.tube.vertices, this.tube.uv, this.tube.faces);
    mgl.popMatrix();
  }

  _makeTubes(alpha) {
    const t = this.time;
    const pulse1 = new SinPulse(40, 60, 2.14, 0);
    const pulseU = new SinPulse(-10, 20, 0.9, 0);

    for (let v = 0; v < TUBE_V; v++) {
      let r = pulse1.calc(t * v * 0.55);
      for (let u = 0; u < TUBE_U; u++) {
        r += pulseU.calc(t * v * 2 * Math.PI * u / TUBE_U * 0.4);
        const node = this.table[u + v * TUBE_U];
        node.setData(
          r * Math.cos(u * 2 * Math.PI / TUBE_U),
          v * 140 - (TUBE_V - 1) * 70.0,
          r * Math.sin(u * 2 * Math.PI / TUBE_U),
          u / TUBE_U + t * 0.1,
          (v / TUBE_V) * 1.5 - t * 0.3,
          1, 1, 1, 0.6 * alpha,
        );
      }
    }
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = this.time = time - timeStart;
    const aspect = 480 / 640;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-0.5, 0.5, -0.5 * aspect, 0.5 * aspect, 1.0, 1000.0);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    let alpha;
    if (t > CHANGE_TIME1) {
      alpha = clamp01(t - CHANGE_TIME1);
      if (t > CHANGE_TIME2) {
        alpha = clamp01(t - CHANGE_TIME2);
        if (t > CHANGE_TIME3) {
          alpha = clamp01(alpha * (1 + CHANGE_TIME3 - t));
        }
      } else {
        alpha = clamp01(alpha * (CHANGE_TIME2 - t));
      }
    } else {
      alpha = clamp01(CHANGE_TIME1 - t);
    }

    mgl.color4(1, 1, 1, 0.15 * alpha);

    mgl.enableTexture(true);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.bindTexture(this.tex2);

    mgl.begin(mgl.QUADS);
    mgl.texCoord2(0, 0); mgl.vertex3(-10, 10, -30);
    mgl.texCoord2(1, 0); mgl.vertex3(10, 10, -30);
    mgl.texCoord2(1, 1); mgl.vertex3(10, -10, -30);
    mgl.texCoord2(0, 1); mgl.vertex3(-10, -10, -30);
    mgl.end();

    mgl.translate(0, 0, -420);
    mgl.rotate(40 * Math.sin(t / 2), 1, 0, 0);
    mgl.rotate(30 + 20 * Math.sin(t / 2), 0, 0, 1);

    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.depthFunc(mgl.LEQUAL);

    mgl.enableCullFace(false);

    this.sobject.enableNormals();
    this._makeTubes(alpha);

    mgl.color4(1, 1, 1, (Math.sin(t * 5) * 0.05 + 0.15) * alpha);

    if (t > CHANGE_TIME1) {
      if (t > CHANGE_TIME2) {
        mgl.rotate(90, 0, 0, 1);
        mgl.rotate(180, 1, 0, 0);
      } else {
        mgl.rotate(90, 0, 0, 1);
      }
    }

    for (let x = 0; x < 8; x++) {
      this._renderTube(1 + x * 0.4, 10 * t * x);
    }

    this.sobject.render(mgl);

    mgl.matrixMode(mgl.TEXTURE);
    mgl.loadIdentity();
    mgl.matrixMode(mgl.MODELVIEW);
  }
}
