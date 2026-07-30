// FFDEnv (67.7 - 87s): the metaball-like morph — cycles through the six
// objects of meta.3ds, blending vertices, with a UV-textured pass plus an
// additive camera-space environment pass, an overlay and scanline flicker.

import { darkQuads } from './common.js';
import { clamp01, Vec3 } from '../mathlib.js';

const CHANGE_TIME3 = 18.3;

export class FFDEnv {
  constructor(mgl, tex, scene) {
    this.mgl = mgl;
    this.tex1 = tex.loadTexture('data/textures/max_t3.jpg');
    this.tex2 = tex.loadTexture('data/textures/gothickiemura02.jpg');
    this.texBlend = tex.loadTexture('data/textures/blend.tga');
    this.scene = scene;

    const obj = scene.objects[0];
    this.faces = obj.faces;
    this.uv = obj.uv;
    this.nVertices = obj.nvertices;
    this.vertices = new Float32Array(this.nVertices * 3);
    this.normals = new Float32Array(this.nVertices * 3);
    this.envUV = new Float32Array(this.nVertices * 2);
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = time - timeStart;

    const alpha = clamp01(1 - t + CHANGE_TIME3);

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-0.6, 0.6, -0.45, 0.45, 1, 1000);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableTexture(true);
    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);
    mgl.enableCullFace(true);
    mgl.enableDepthTest(true);
    mgl.depthMask(true);
    mgl.cullFace(mgl.FRONT);
    mgl.depthFunc(mgl.LEQUAL);

    mgl.translate(0, 0, -30);
    mgl.rotate(15 * Math.sin(t), 0, 1, 0);
    mgl.rotate(t * 45, 0, 0, 1);
    mgl.rotate(90, 1, 0, 0);

    const camRot = mgl.getModelView();
    camRot.setBaseW(0, 0, 0, 1);

    // morph between consecutive objects of the scene
    let fT = (t * 0.5) % 1;
    fT = Math.cos((1 - fT) * 3.14159) * 0.5 + 0.5;

    const nObjects = 6;
    const iO1 = Math.floor((t * 0.5) % nObjects);
    const iO2 = iO1 + 1 >= nObjects ? 0 : iO1 + 1;

    const o1 = this.scene.objects[iO1];
    const o2 = this.scene.objects[iO2];

    const v = this.vertices, n = this.normals;
    const nVerts = Math.min(this.nVertices, o1.nvertices, o2.nvertices);
    for (let i = 0; i < nVerts * 3; i++) {
      v[i] = (o2.vlocal[i] - o1.vlocal[i]) * fT + o1.vlocal[i];
      n[i] = (o2.wlocal[i] - o1.wlocal[i]) * fT + o1.wlocal[i];
    }

    // camera-space env mapping from morphed normals
    const cN = new Vec3(), tmp = new Vec3();
    for (let i = 0; i < nVerts; i++) {
      tmp.set(n[i * 3], n[i * 3 + 1], n[i * 3 + 2]);
      camRot.mulPoint(tmp, cN);
      this.envUV[i * 2] = cN.x * 0.5 + 0.5;
      this.envUV[i * 2 + 1] = cN.y * 0.5 + 0.5;
    }

    // mesh pass
    mgl.bindTexture(this.tex2);
    mgl.color4(1, 1, 1, alpha);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);
    mgl.drawElements(this.vertices, this.uv, this.faces);

    // env pass
    mgl.bindTexture(this.tex1);
    mgl.color4(1, 1, 1, alpha);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.drawElements(this.vertices, this.envUV, this.faces);

    // fullscreen blend overlay
    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.ortho(0, 1, 0, 1, -1, 1);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableCullFace(false);

    mgl.bindTexture(this.texBlend);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);
    mgl.color4(1, 1, 1, alpha);
    mgl.begin(mgl.QUADS);
    mgl.texCoord2(0, 0); mgl.vertex3(0, 0, 0);
    mgl.texCoord2(1, 0); mgl.vertex3(1, 0, 0);
    mgl.texCoord2(1, 1); mgl.vertex3(1, 1, 0);
    mgl.texCoord2(0, 1); mgl.vertex3(0, 1, 0);
    mgl.end();

    darkQuads(mgl, Math.floor(Math.sin(t) * 40 + 50), [0, 0, 0, 0.3 * Math.sin(t)]);
  }
}
