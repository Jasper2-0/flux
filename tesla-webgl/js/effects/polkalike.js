// PolkaLike (165 - 203s): four winding tracks of billboarded "polka" sprites
// streaming toward the camera over a tunnel of rotating background quads.
// Track state advances with a fixed 60 Hz step, as the original assumed.

import { Vec3 } from '../mathlib.js';
import { clamp01 } from '../mathlib.js';

const Z_MAX = 300;
const CHANGE_TIME = 10;
const CHANGE_TIME1 = 20;
const CHANGE_TIME3 = 19;

const STEP = 0.016;

class TrackElem {
  constructor() {
    this.pos = new Vec3();
    this.delPos = new Vec3();
    this.angleZ = 0;
  }
  init(pos, delPos, angleZ) {
    this.pos.copy(pos);
    this.delPos.copy(delPos);
    this.angleZ = angleZ;
  }
  update(dt) {
    this.pos.x += this.delPos.x * dt;
    this.pos.y += this.delPos.y * dt;
    this.pos.z += this.delPos.z * dt;
    this.angleZ += -2 * 1.234 * dt;
  }
}

class Track {
  constructor(nElems, initAngleChange, initPosChange) {
    this.elems = [];
    this.initAngleZ = 0;
    this.initAngleChange = initAngleChange;
    this.initPos = 0;
    this.initPosChange = initPosChange;
    this.initPosVec = new Vec3();

    const delPos = new Vec3(0, 0, 100);
    for (let i = 0; i < nElems; i++) {
      this.initPosVec.x = 10 * Math.cos(this.initPos) + 10 * Math.sin(this.initPos * 1.32);
      this.initPosVec.y = 15 * Math.sin(this.initPos) + 5 * Math.sin(this.initPos * 0.89);
      this.initPosVec.z = -3 * Z_MAX - (Z_MAX / nElems) * i;

      const e = new TrackElem();
      e.init(this.initPosVec, delPos, this.initAngleZ);
      this.elems.push(e);

      this.initAngleZ += this.initAngleChange * 0.042;
      this.initPos += this.initPosChange * 0.042;
    }
  }

  // one fixed simulation step (state part of the original Render)
  step(dt, time) {
    for (const e of this.elems) {
      e.update(dt);
      if (time < CHANGE_TIME3 && e.pos.z > 0) {
        e.pos.z = -Z_MAX;
        e.pos.x = this.initPosVec.x;
        e.pos.y = this.initPosVec.y;
        e.angleZ = this.initAngleZ;
      }
    }

    if (time < CHANGE_TIME) {
      this.initPosVec.x = 10 * Math.cos(this.initPos) + 10 * Math.sin(this.initPos * 1.32);
      this.initPosVec.y = 15 * Math.sin(this.initPos) + 5 * Math.sin(this.initPos * 0.89);
    } else if (time < CHANGE_TIME1) {
      this.initPosVec.x = 15 * Math.cos(this.initPos) + 15 * Math.sin(this.initPos * 2.32);
      this.initPosVec.y = 30 * Math.sin(this.initPos) + 15 * Math.sin(this.initPos * 1.89);
    } else {
      this.initPosVec.x = 30 * Math.cos(this.initPos) + 15 * Math.sin(this.initPos * 0.72);
      this.initPosVec.y = 25 * Math.sin(this.initPos) + 19 * Math.sin(this.initPos * 0.59);
    }
    this.initPosVec.z = -Z_MAX;

    this.initAngleZ += this.initAngleChange * dt;
    this.initPos += this.initPosChange * dt;
  }

  render(mgl) {
    mgl.begin(mgl.QUADS);
    for (const e of this.elems) {
      const cx = Math.cos(e.angleZ);
      const sy = Math.sin(e.angleZ);
      const rot1x = sy, rot1y = -cx;
      const rotx = cx * 5, roty = sy * 5;

      const alpha = -e.pos.z / Z_MAX + 0.2;
      mgl.color4(1, 1, 1, alpha);

      mgl.texCoord2(0, 0);
      mgl.vertex3(e.pos.x + rotx - rot1x, e.pos.y + roty - rot1y, e.pos.z);
      mgl.texCoord2(1, 0);
      mgl.vertex3(e.pos.x + rotx + rot1x, e.pos.y + roty + rot1y, e.pos.z);
      mgl.texCoord2(1, 1);
      mgl.vertex3(e.pos.x - rotx + rot1x, e.pos.y - roty + rot1y, e.pos.z);
      mgl.texCoord2(0, 1);
      mgl.vertex3(e.pos.x - rotx - rot1x, e.pos.y - roty - rot1y, e.pos.z);
    }
    mgl.end();
  }
}

export class PolkaLike {
  constructor(mgl, tex) {
    this.mgl = mgl;
    this.texture = tex.loadTexture('data/textures/polka.png');
    this.back = tex.loadTexture('data/textures/max_t3.jpg');

    const pi = Math.PI;
    this.tracks = [
      new Track(128, pi / 2.13, pi / 3.41),
      new Track(128, pi / 2.93, pi / 1.72),
      new Track(128, pi / 3.97, pi),
      new Track(128, pi * 1.23, pi * 2.17),
    ];
    this.simTime = 0;
  }

  do(time, timeStart) {
    const mgl = this.mgl;
    const t = time - timeStart;

    mgl.matrixMode(mgl.PROJECTION);
    mgl.loadIdentity();
    mgl.frustum(-0.6, 0.6, -0.45, 0.45, 1, Z_MAX);

    mgl.matrixMode(mgl.MODELVIEW);
    mgl.loadIdentity();

    mgl.enableBlend(true);
    mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE);
    mgl.enableTexture(true);
    mgl.enableDepthTest(false);
    mgl.depthMask(false);
    mgl.enableCullFace(false);

    mgl.bindTexture(this.back);

    const fadeIn = clamp01(t * 0.5);
    const col = 0.25 * Math.sin(t) + 0.5;
    mgl.color4(1, 1, 1, col * fadeIn);

    for (let a = 0; a < 16; a++) {
      mgl.rotate(4 * Math.sin(t), 0, 1, 0);
      mgl.rotate(5 * t, 0, 0, 1);
      const z = -1.4 - a;
      mgl.begin(mgl.QUADS);
      mgl.texCoord2(0, 0); mgl.vertex3(-1, 1, z);
      mgl.texCoord2(1, 0); mgl.vertex3(1, 1, z);
      mgl.texCoord2(1, 1); mgl.vertex3(1, -1, z);
      mgl.texCoord2(0, 1); mgl.vertex3(-1, -1, z);
      mgl.end();
    }

    // catch the simulation up with fixed steps, then draw
    while (this.simTime < t) {
      for (const track of this.tracks) track.step(STEP, this.simTime);
      this.simTime += STEP;
    }

    mgl.bindTexture(this.texture);
    for (const track of this.tracks) track.render(mgl);
  }
}
