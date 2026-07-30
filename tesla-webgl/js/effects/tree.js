// Tree (186 - 201s): the kbuu2.3ds mesh run through a 5x5x5 Catmull-Rom
// free-form deformation lattice that breathes with time, additively textured
// with a scrolling texture matrix. Layered over PolkaLike.

import { FFD } from './common.js';
import { clamp01 } from '../mathlib.js';

const CHANGE_TIME3 = 14;

export class Tree {
  constructor(mgl, tex, scene) {
    this.mgl = mgl;
    this.scene = scene;

    const obj = scene.objects[0];
    this.nVertices = obj.nvertices;
    this.faces = obj.faces;
    this.vertices = new Float32Array(this.nVertices * 3);

    // the original reinterprets the vertex normal (wlocal) as the UV map here
    this.uv = new Float32Array(this.nVertices * 2);
    for (let i = 0; i < this.nVertices; i++) {
      this.uv[i * 2] = obj.wlocal[i * 3];
      this.uv[i * 2 + 1] = obj.wlocal[i * 3 + 1];
    }

    this.tex1 = tex.loadTexture('data/textures/max_t3.jpg');

    this.ffd = new FFD();
    this.ffd.setVectorTable(obj.vlocal, this.nVertices);
    this.deform = new Float32Array(5 * 5 * 5 * 3);
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = time - timeStart;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-0.6, 0.6, -0.45, 0.45, 1, 1000);

    mgl.matrixMode(mgl.TEXTURE);
    mgl.loadIdentity();

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableTexture(true);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);
    mgl.enableCullFace(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);

    let alpha;
    if (t > CHANGE_TIME3) {
      alpha = 1 + CHANGE_TIME3 - t;
    } else {
      alpha = clamp01(t * 0.5);
    }

    mgl.color4(1, 1, 1, 0.2 * alpha);

    mgl.translate(0, 0, -12);
    mgl.rotate(5 * Math.sin(t / 3), 0, 0, 1);
    mgl.rotate(t * 10, 0, 1, 0);

    // animate the FFD lattice
    for (let z = 0; z < 5; z++) {
      for (let y = 0; y < 5; y++) {
        for (let x = 0; x < 5; x++) {
          const i = (z * 25 + y * 5 + x) * 3;
          this.deform[i] = (x * 4 - 8) * (Math.sin(t * 1.33 + y * 4) * 0.2 + 0.8);
          this.deform[i + 1] = (y * 4 - 8) * (Math.cos(t * 2 + x * 10) * 0.2 + 0.8);
          this.deform[i + 2] = (z * 4 - 8) * (Math.cos(t * 2.33 + y * 10) * 0.2 + 0.8);
        }
      }
    }

    this.ffd.setDeform(this.deform, 5, 5, 5);
    this.ffd.calcSplineDeform(this.vertices);

    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);

    mgl.matrixMode(mgl.TEXTURE);
    mgl.loadIdentity();
    mgl.translate(t * 0.1, t * 0.5, 0);

    mgl.bindTexture(this.tex1);
    mgl.drawElements(this.vertices, this.uv, this.faces);

    mgl.loadIdentity();
    mgl.matrixMode(mgl.MODELVIEW);
  }
}
