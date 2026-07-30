// Shared helpers used by several Tesla effects.

import { rand } from '../mathlib.js';

// CDarkQuads: random full-width dark horizontal bands in a 640x480 ortho view.
export function darkQuads(mgl, seed, color = [0, 0, 0, 0.2]) {
  mgl.matrixMode(mgl.PROJECTION);
  mgl.loadIdentity();
  mgl.ortho(0, 640, 0, 480, -1, 1);

  mgl.matrixMode(mgl.MODELVIEW);
  mgl.loadIdentity();

  mgl.enableTexture(false);
  mgl.enableDepthTest(false);
  mgl.enableCullFace(false);
  mgl.blendFunc(mgl.SRC_ALPHA, mgl.ONE_MINUS_SRC_ALPHA);

  mgl.color4(color[0], color[1], color[2], color[3]);

  mgl.begin(mgl.QUADS);
  for (let i = 0; i < seed; i++) {
    const y = rand() % 480;
    const h = rand() % 8 + 1;
    mgl.vertex3(0, y + h, 0);
    mgl.vertex3(640, y + h, 0);
    mgl.vertex3(640, y - h, 0);
    mgl.vertex3(0, y - h, 0);
  }
  mgl.end();
}

// Lat/long sphere with the demo's exact vertex/face layout (MakeSphere).
export function makeSphere(radius, vSeg, hSeg) {
  const nVerts = vSeg * hSeg;
  const nFaces = (vSeg - 1) * (hSeg - 1) * 2;
  const vertices = new Float32Array(nVerts * 3);
  const uv = new Float32Array(nVerts * 2);
  const faces = new Uint32Array(nFaces * 3);

  for (let v = 0; v < vSeg; v++) {
    for (let h = 0; h < hSeg; h++) {
      const i = v * hSeg + h;
      vertices[i * 3] = radius * Math.cos(Math.PI * h * 2 / (hSeg - 1)) * Math.sin(Math.PI * v / (vSeg - 1));
      vertices[i * 3 + 1] = radius * Math.cos(Math.PI * v / (vSeg - 1));
      vertices[i * 3 + 2] = radius * Math.sin(Math.PI * h * 2 / (hSeg - 1)) * Math.sin(Math.PI * v / (vSeg - 1));
      uv[i * 2] = h / (hSeg - 1);
      uv[i * 2 + 1] = v / (vSeg - 1);
    }
  }

  for (let v = 0; v < vSeg - 1; v++) {
    for (let h = 0; h < hSeg - 1; h++) {
      const f = 6 * (v * (hSeg - 1) + h);
      faces[f] = v * hSeg + h;
      faces[f + 1] = v * hSeg + h + 1;
      faces[f + 2] = (v + 1) * hSeg + h;
      faces[f + 3] = v * hSeg + h + 1;
      faces[f + 4] = (v + 1) * hSeg + h + 1;
      faces[f + 5] = (v + 1) * hSeg + h;
    }
  }

  return { vertices, uv, faces };
}

// Open cylinder ("tube") as used by CTubes / CFaceMorph backgrounds.
export function makeTube(radius, vSeg, hSeg) {
  const nVerts = vSeg * hSeg;
  const nFaces = (vSeg - 1) * (hSeg - 1) * 2;
  const vertices = new Float32Array(nVerts * 3);
  const uv = new Float32Array(nVerts * 2);
  const faces = new Uint32Array(nFaces * 3);

  for (let v = 0; v < vSeg; v++) {
    for (let h = 0; h < hSeg; h++) {
      const i = v * hSeg + h;
      vertices[i * 3] = radius * Math.cos(Math.PI * h * 2 / (hSeg - 1));
      vertices[i * 3 + 1] = v * 70 - (vSeg - 1) * 35;
      vertices[i * 3 + 2] = radius * Math.sin(Math.PI * h * 2 / (hSeg - 1));
      uv[i * 2] = h / (hSeg - 1);
      uv[i * 2 + 1] = v / (vSeg - 1);
    }
  }

  for (let v = 0; v < vSeg - 1; v++) {
    for (let h = 0; h < hSeg - 1; h++) {
      const f = 6 * (v * (hSeg - 1) + h);
      faces[f] = v * hSeg + h;
      faces[f + 1] = v * hSeg + h + 1;
      faces[f + 2] = (v + 1) * hSeg + h;
      faces[f + 3] = v * hSeg + h + 1;
      faces[f + 4] = (v + 1) * hSeg + h + 1;
      faces[f + 5] = (v + 1) * hSeg + h;
    }
  }

  return { vertices, uv, faces };
}

// Piecewise-linear envelope over (time, value) pairs; returns 0 outside.
export function keyVal(keys, t) {
  for (let i = 0; i < keys.length - 1; i++) {
    if (t >= keys[i][0] && t < keys[i + 1][0]) {
      return (keys[i + 1][1] - keys[i][1]) * ((t - keys[i][0]) / (keys[i + 1][0] - keys[i][0])) + keys[i][1];
    }
  }
  return 0;
}

export function catmull(a, b, c, d, t) {
  const fa = 0.5 * (3 * b + d - a - 3 * c);
  const fb = a + 2 * c - 0.5 * (5 * b + d);
  const fc = 0.5 * (c - a);
  return t * t * t * fa + t * t * fb + t * fc + b;
}

// CFFD: free-form deformation over a lattice, Catmull-Rom spline variant.
// vectors: Float32Array xyz source points; deform: lattice points as
// Float32Array xyz of resX*resY*resZ entries; dest: Float32Array xyz output.
export class FFD {
  constructor() {
    this.vectors = null;
    this.nElements = 0;
    this.deform = null;
    this.resX = this.resY = this.resZ = 0;
    this.bmin = [0, 0, 0];
    this.bmax = [0, 0, 0];
  }

  setVectorTable(vectors, nElems) {
    this.vectors = vectors;
    this.nElements = nElems;
  }

  setDeform(deform, resX, resY, resZ) {
    this.deform = deform;
    this.resX = resX; this.resY = resY; this.resZ = resZ;
  }

  _calcBBox() {
    const v = this.vectors;
    const bmin = [1e30, 1e30, 1e30];
    const bmax = [-1e30, -1e30, -1e30];
    for (let i = 0; i < this.nElements; i++) {
      for (let c = 0; c < 3; c++) {
        const val = v[i * 3 + c];
        if (val < bmin[c]) bmin[c] = val;
        if (val > bmax[c]) bmax[c] = val;
      }
    }
    for (let c = 0; c < 3; c++) { bmin[c] -= 0.0001; bmax[c] += 0.0001; }
    this.bmin = bmin; this.bmax = bmax;
  }

  _point(x, y, z, out) {
    if (x < 0) x = 0; else if (x >= this.resX) x = this.resX - 1;
    if (y < 0) y = 0; else if (y >= this.resY) y = this.resY - 1;
    if (z < 0) z = 0; else if (z >= this.resZ) z = this.resZ - 1;
    const i = (x + (y + z * this.resY) * this.resX) * 3;
    out[0] = this.deform[i]; out[1] = this.deform[i + 1]; out[2] = this.deform[i + 2];
  }

  calcSplineDeform(dest) {
    this._calcBBox();
    const delX = 1.0 / ((this.bmax[0] - this.bmin[0]) / (this.resX - 1));
    const delY = 1.0 / ((this.bmax[1] - this.bmin[1]) / (this.resY - 1));
    const delZ = 1.0 / ((this.bmax[2] - this.bmin[2]) / (this.resZ - 1));

    const p = [new Float32Array(3), new Float32Array(3), new Float32Array(3), new Float32Array(3)];
    const aI1 = [];
    for (let i = 0; i < 4; i++) {
      aI1.push([new Float32Array(3), new Float32Array(3), new Float32Array(3), new Float32Array(3)]);
    }
    const aI2 = [new Float32Array(3), new Float32Array(3), new Float32Array(3), new Float32Array(3)];

    for (let s = 0; s < this.nElements; s++) {
      const tx = (this.vectors[s * 3] - this.bmin[0]) * delX;
      const ty = (this.vectors[s * 3 + 1] - this.bmin[1]) * delY;
      const tz = (this.vectors[s * 3 + 2] - this.bmin[2]) * delZ;
      const nx = Math.floor(tx), ny = Math.floor(ty), nz = Math.floor(tz);
      const fx = tx - nx, fy = ty - ny, fz = tz - nz;

      for (let q = 0; q < 4; q++) {
        for (let i = 0; i < 4; i++) {
          this._point(nx - 1, ny - 1 + i, nz - 1 + q, p[0]);
          this._point(nx, ny - 1 + i, nz - 1 + q, p[1]);
          this._point(nx + 1, ny - 1 + i, nz - 1 + q, p[2]);
          this._point(nx + 2, ny - 1 + i, nz - 1 + q, p[3]);
          for (let c = 0; c < 3; c++) {
            aI1[i][q][c] = catmull(p[0][c], p[1][c], p[2][c], p[3][c], fx);
          }
        }
      }

      for (let i = 0; i < 4; i++) {
        for (let c = 0; c < 3; c++) {
          aI2[i][c] = catmull(aI1[0][i][c], aI1[1][i][c], aI1[2][i][c], aI1[3][i][c], fy);
        }
      }

      for (let c = 0; c < 3; c++) {
        dest[s * 3 + c] = catmull(aI2[0][c], aI2[1][c], aI2[2][c], aI2[3][c], fz);
      }
    }
  }
}
