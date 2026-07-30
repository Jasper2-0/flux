// FaceMorph (203 - 222.5s): morphing head meshes from faces.3ds with two
// scrolling UV-textured passes plus an additive camera-space env pass, over a
// background of rotating textured cylinders, finished with a blend overlay.

import { makeTube } from './common.js';
import { clamp01, Vec3 } from '../mathlib.js';

const MORPH_SPEED = 0.5;
const MORPH_OBJ = [1, 0, 3, 0];
const CHANGE_TIME3 = 19.5;

export class FaceMorph {
  constructor(mgl, tex, scene) {
    this.mgl = mgl;
    this.tex1 = tex.loadTexture('data/textures/y6.jpg');
    this.tex1a = tex.loadTexture('data/textures/t1a.jpg');
    this.tex2 = tex.loadTexture('data/textures/max_t1.jpg');
    this.backTex = tex.loadTexture('data/textures/y7.jpg');
    this.texBlend = tex.loadTexture('data/textures/blend2.png');
    this.scene = scene;

    const obj = scene.objects[2];
    this.faces = obj.faces;
    this.uv = obj.uv;
    this.nVertices = obj.nvertices;
    this.vertices = new Float32Array(this.nVertices * 3);
    this.normals = new Float32Array(this.nVertices * 3);
    this.envUV = new Float32Array(this.nVertices * 2);

    this.tube = makeTube(90, 8, 8);
  }

  _renderTube(scale, angle0) {
    const mgl = this.mgl;
    mgl.enableTexture(true);
    mgl.bindTexture(this.backTex);
    mgl.pushMatrix();
    mgl.rotate(angle0, 0, 1, 0);
    mgl.scale(scale, scale, scale);
    mgl.drawElements(this.tube.vertices, this.tube.uv, this.tube.faces);
    mgl.popMatrix();
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = time - timeStart;

    const alpha = clamp01(CHANGE_TIME3 - t);

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-12.6, 12.6, -12.45, 12.45, 1, 1000);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.translate(2, 0, -13);
    mgl.rotate(20 * Math.sin(t), 1, 0, 0);
    mgl.rotate(10 * Math.sin(t * 0.9) - 15, 0, 1, 0);
    mgl.rotate(180, 0, 1, 0);

    const cam = mgl.getModelView();
    cam.setBaseW(0, 0, 0, 1);

    mgl.enableTexture(true);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.enableCullFace(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);

    // background tubes
    mgl.color4(1, 1, 1, 0.4 * alpha);
    for (let u = 0; u < 4; u++) {
      this._renderTube(1 + u, 1 + t * 10 * u);
    }

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-0.6, 0.6, -0.45, 0.45, 1, 1000);

    mgl.enableCullFace(true);
    mgl.cullFace(mgl.FRONT);
    mgl.enableDepthTest(true);
    mgl.depthMask(true);
    mgl.depthFunc(mgl.LEQUAL);

    // morphing: uses the world-space (vglobal) vertices, as the original
    // overwrote vlocal with vglobal at load time
    let fT = (t * MORPH_SPEED) % 1;
    fT = Math.cos((1 - fT) * 3.14159) * 0.5 + 0.5;

    let iO1 = Math.floor((t * MORPH_SPEED) % 4);
    let iO2 = iO1 + 1 >= 4 ? 0 : iO1 + 1;
    iO1 = MORPH_OBJ[iO1];
    iO2 = MORPH_OBJ[iO2];

    const o1 = this.scene.objects[iO1];
    const o2 = this.scene.objects[iO2];

    const v = this.vertices, n = this.normals;
    const nVerts = Math.min(this.nVertices, o1.nvertices, o2.nvertices);
    const cN = new Vec3(), tmp = new Vec3();
    for (let i = 0; i < nVerts; i++) {
      for (let c = 0; c < 3; c++) {
        v[i * 3 + c] = (o2.vglobal[i * 3 + c] - o1.vglobal[i * 3 + c]) * fT + o1.vglobal[i * 3 + c];
        n[i * 3 + c] = (o2.wlocal[i * 3 + c] - o1.wlocal[i * 3 + c]) * fT + o1.wlocal[i * 3 + c];
      }
      tmp.set(n[i * 3], n[i * 3 + 1], n[i * 3 + 2]);
      cam.mulPoint(tmp, cN);
      this.envUV[i * 2] = cN.x * 0.5 + 0.5;
      this.envUV[i * 2 + 1] = cN.y * 0.5 + 0.5;
    }

    // mesh passes with animated texture matrix
    mgl.matrixMode(mgl.TEXTURE);
    mgl.color4(1, 1, 1, alpha);
    mgl.bindTexture(this.tex1);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);
    mgl.loadIdentity();
    mgl.translate(t * 0.05, t * 0.1, 0);
    mgl.rotate(t * 30, 0, 0, 1);
    mgl.drawElements(this.vertices, this.uv, this.faces);

    mgl.color4(1, 1, 1, 0.5 * alpha);
    mgl.bindTexture(this.tex1a);
    mgl.loadIdentity();
    mgl.translate(t * 0.05, -t * 0.05, 0);
    mgl.rotate(-t * 20, 0, 0, 1);
    mgl.drawElements(this.vertices, this.uv, this.faces);
    mgl.loadIdentity();

    // env pass
    mgl.color4(1, 1, 1, alpha);
    mgl.bindTexture(this.tex2);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.drawElements(this.vertices, this.envUV, this.faces);

    mgl.loadIdentity();
    mgl.matrixMode(mgl.MODELVIEW);

    // fullscreen blend overlay
    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.ortho(0, 1, 0, 1, -1, 1);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableCullFace(false);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);

    mgl.bindTexture(this.texBlend);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);
    mgl.color4(1, 1, 1, alpha);
    mgl.begin(mgl.QUADS);
    mgl.texCoord2(0, 0); mgl.vertex3(0, 0, 0);
    mgl.texCoord2(1, 0); mgl.vertex3(1, 0, 0);
    mgl.texCoord2(1, 1); mgl.vertex3(1, 1, 0);
    mgl.texCoord2(0, 1); mgl.vertex3(0, 1, 0);
    mgl.end();
  }
}
