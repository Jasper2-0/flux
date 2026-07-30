// Catmull-Rom spline-patch tessellator (CSplinedObject / CObjectNode port).
// A grid of control nodes (optionally wrapped in U) is subdivided per patch
// into seedU x seedV vertices, each carrying position, normal-derived env UV,
// texture UV and color; patches are rendered immediately with one or more
// texture layers (material 1 = UV-mapped, material 2 = env-mapped).

import { catmull } from './common.js';
import { Vec3 } from '../mathlib.js';

class NodeData {
  constructor() {
    this.px = 0; this.py = 0; this.pz = 0;
    this.nx = 0; this.ny = 0;
    this.u = 0; this.v = 0;
    this.r = 1; this.g = 1; this.b = 1; this.a = 1;
  }
  copyFrom(o) {
    this.px = o.px; this.py = o.py; this.pz = o.pz;
    this.nx = o.nx; this.ny = o.ny;
    this.u = o.u; this.v = o.v;
    this.r = o.r; this.g = o.g; this.b = o.b; this.a = o.a;
  }
}

export class ObjectNode {
  constructor() {
    this.up = this.down = this.left = this.right = null;
    this.wrapUV = 0;
    this.data = new NodeData();
    this.normal0 = new Vec3();
    this.normal1 = new Vec3();
  }

  setData(px, py, pz, u, v, r, g, b, a) {
    const d = this.data;
    d.px = px; d.py = py; d.pz = pz;
    d.u = u; d.v = v;
    d.r = r; d.g = g; d.b = b; d.a = a;
  }

  validLeft() { return this.left || this; }
  validRight() { return this.right || this; }
  validUp() { return this.up || this; }
  validDown() { return this.down || this; }
}

function sInterpolateNodes(dest, n1, n2, n3, n4, t) {
  const d1 = n1.data, d2 = n2.data, d3 = n3.data, d4 = n4.data;
  dest.px = catmull(d1.px, d2.px, d3.px, d4.px, t);
  dest.py = catmull(d1.py, d2.py, d3.py, d4.py, t);
  dest.pz = catmull(d1.pz, d2.pz, d3.pz, d4.pz, t);
  dest.u = catmull(d1.u, d2.u, d3.u, d4.u, t);
  dest.v = catmull(d1.v, d2.v, d3.v, d4.v, t);
  dest.nx = catmull(d1.nx, d2.nx, d3.nx, d4.nx, t);
  dest.ny = catmull(d1.ny, d2.ny, d3.ny, d4.ny, t);
  dest.r = catmull(d1.r, d2.r, d3.r, d4.r, t);
  dest.g = catmull(d1.g, d2.g, d3.g, d4.g, t);
  dest.b = catmull(d1.b, d2.b, d3.b, d4.b, t);
  dest.a = catmull(d1.a, d2.a, d3.a, d4.a, t);
}

function sInterpolateData(dest, e1, e2, e3, e4, t, wrap) {
  dest.px = catmull(e1.px, e2.px, e3.px, e4.px, t);
  dest.py = catmull(e1.py, e2.py, e3.py, e4.py, t);
  dest.pz = catmull(e1.pz, e2.pz, e3.pz, e4.pz, t);
  dest.nx = catmull(e1.nx, e2.nx, e3.nx, e4.nx, t);
  dest.ny = catmull(e1.ny, e2.ny, e3.ny, e4.ny, t);
  if (wrap === 0) dest.u = catmull(e1.u, e2.u, e3.u, e4.u, t);
  else if (wrap > 0) dest.u = catmull(e1.u, e2.u, e3.u + 1, e4.u + 1, t);
  else dest.u = catmull(e1.u - 1, e2.u, e3.u, e4.u, t);
  dest.v = catmull(e1.v, e2.v, e3.v, e4.v, t);
  dest.r = catmull(e1.r, e2.r, e3.r, e4.r, t);
  dest.g = catmull(e1.g, e2.g, e3.g, e4.g, t);
  dest.b = catmull(e1.b, e2.b, e3.b, e4.b, t);
  dest.a = catmull(e1.a, e2.a, e3.a, e4.a, t);
}

export class SplinedObject {
  constructor() {
    this.root = null;
    this.nodes = null;
    this.resU = 0; this.resV = 0;
    this.seedU = 4; this.seedV = 4;
    this.multiTex = [];
    this.generateNormals = false;

    this._edges = [[], [], [], []];
    this._hline1 = [];
    this._hline2 = [];
    for (let i = 0; i < 64; i++) {
      for (let e = 0; e < 4; e++) this._edges[e].push(new NodeData());
      this._hline1.push(new NodeData());
      this._hline2.push(new NodeData());
    }
  }

  setSeed(u, v) {
    this.seedU = Math.min(u, 63);
    this.seedV = Math.min(v, 63);
  }

  setMultiTexData(list) { this.multiTex = list; }

  addTable(nodes, resU, resV) {
    this.nodes = nodes;
    this.resU = resU;
    this.resV = resV;
    this.root = nodes[0];
    for (let v = 0; v < resV; v++) {
      for (let u = 0; u < resU; u++) {
        const n = nodes[v * resU + u];
        if (u > 0) { n.left = nodes[v * resU + u - 1]; n.left.right = n; }
        if (v > 0) { n.up = nodes[(v - 1) * resU + u]; n.up.down = n; }
      }
    }
  }

  closeTableU() {
    for (let v = 0; v < this.resV; v++) {
      const first = this.nodes[v * this.resU];
      const last = this.nodes[v * this.resU + this.resU - 1];
      last.right = first;
      first.left = last;
      last.wrapUV = 1;
    }
  }

  enableNormals() { this.generateNormals = true; }

  _generateNormals(modelView) {
    const right = new Vec3(), down = new Vec3(), rightDown = new Vec3();
    for (let v = 0; v < this.resV; v++) {
      for (let u = 0; u < this.resU; u++) {
        const n = this.nodes[u + v * this.resU];
        const p = n.data;
        const pr = n.validRight().data;
        const pd = n.validDown().data;
        const prd = n.validRight().validDown().data;
        right.set(pr.px - p.px, pr.py - p.py, pr.pz - p.pz);
        down.set(pd.px - p.px, pd.py - p.py, pd.pz - p.pz);
        rightDown.set(prd.px - p.px, prd.py - p.py, prd.pz - p.pz);
        // note: the original's Cross() uses "+" everywhere (a quirk kept here)
        n.normal1.set(
          rightDown.y * right.z + rightDown.z * right.y,
          rightDown.z * right.x + rightDown.x * right.z,
          rightDown.x * right.y + rightDown.y * right.x);
        n.normal0.set(
          rightDown.y * down.z + rightDown.z * down.y,
          rightDown.z * down.x + rightDown.x * down.z,
          rightDown.x * down.y + rightDown.y * down.x);
        n.normal0.x = -n.normal0.x; n.normal0.y = -n.normal0.y;
        n.normal1.x = -n.normal1.x; n.normal1.y = -n.normal1.y;
      }
    }

    const sum = new Vec3(), tmp = new Vec3();
    for (let v = 0; v < this.resV; v++) {
      for (let u = 0; u < this.resU; u++) {
        const n = this.nodes[u + v * this.resU];
        sum.set(0, 0, 0);
        sum.addSelf(n.normal0);
        sum.addSelf(n.normal1);
        sum.addSelf(n.validUp().normal0);
        sum.addSelf(n.validDown().normal1);
        sum.addSelf(n.validLeft().validUp().normal0);
        sum.addSelf(n.validLeft().validUp().normal1);
        modelView.mulDir(sum, tmp);
        tmp.normalize();
        n.data.nx = tmp.x * 0.5 + 0.5;
        n.data.ny = tmp.y * 0.5 + 0.5;
      }
    }
  }

  render(mgl) {
    if (!this.root) return;

    if (this.generateNormals) {
      this._generateNormals(mgl.getModelView());
    }

    mgl.enableTexture(true);

    let edge = this.root;
    do {
      let node = edge;
      do {
        this._renderPatch(mgl, node);
        node = node.right;
      } while (node && node.right && node !== edge);
      edge = edge.down;
    } while (edge && edge.down);
  }

  _renderPatch(mgl, node) {
    const seedU = this.seedU, seedV = this.seedV;
    const edges = this._edges;

    // edge 1: left column
    let pN2 = node.validLeft();
    let pN1 = pN2.validUp();
    let pN3 = pN2.validDown();
    let pN4 = pN3.validDown();
    edges[0][0].copyFrom(pN2.data);
    for (let v = 1; v < seedV - 1; v++) {
      sInterpolateNodes(edges[0][v], pN1, pN2, pN3, pN4, v / (seedV - 1));
    }
    edges[0][seedV - 1].copyFrom(pN3.data);

    // edge 2: this column
    pN1 = node.validUp();
    pN2 = node;
    pN3 = node.validDown();
    pN4 = pN3.validDown();
    edges[1][0].copyFrom(pN2.data);
    for (let v = 1; v < seedV - 1; v++) {
      sInterpolateNodes(edges[1][v], pN1, pN2, pN3, pN4, v / (seedV - 1));
    }
    edges[1][seedV - 1].copyFrom(pN3.data);

    // edge 3: right column
    pN2 = node.validRight();
    pN1 = pN2.validUp();
    pN3 = pN2.validDown();
    pN4 = pN3.validDown();
    edges[2][0].copyFrom(pN2.data);
    for (let v = 1; v < seedV - 1; v++) {
      sInterpolateNodes(edges[2][v], pN1, pN2, pN3, pN4, v / (seedV - 1));
    }
    edges[2][seedV - 1].copyFrom(pN3.data);

    // edge 4: right-right column
    pN2 = pN2.validRight();
    pN1 = pN2.validUp();
    pN3 = pN2.validDown();
    pN4 = pN3.validDown();
    edges[3][0].copyFrom(pN2.data);
    for (let v = 1; v < seedV - 1; v++) {
      sInterpolateNodes(edges[3][v], pN1, pN2, pN3, pN4, v / (seedV - 1));
    }
    edges[3][seedV - 1].copyFrom(pN3.data);

    let wrap;
    if (node.wrapUV) wrap = 1;
    else if (node.validLeft().wrapUV) wrap = -1;
    else wrap = 0;

    let hline1 = this._hline1, hline2 = this._hline2;

    for (let u = 0; u < seedU; u++) {
      sInterpolateData(hline1[u], edges[0][0], edges[1][0], edges[2][0], edges[3][0], u / (seedU - 1), wrap);
    }

    for (let v = 1; v < seedV; v++) {
      for (let u = 0; u < seedU; u++) {
        sInterpolateData(hline2[u], edges[0][v], edges[1][v], edges[2][v], edges[3][v], u / (seedU - 1), wrap);
      }

      for (const texData of this.multiTex) {
        if (!texData.tex) continue;
        mgl.bindTexture(texData.tex);
        mgl.begin(mgl.TRIANGLES);
        const useEnv = !(texData.material & 1) && (texData.material & 2);
        const emit = (d) => {
          if (useEnv) mgl.texCoord2(d.nx, d.ny);
          else mgl.texCoord2(d.u + texData.uShift, d.v + texData.vShift);
          mgl.color4(d.r, d.g, d.b, d.a * texData.alpha);
          mgl.vertex3(d.px, d.py, d.pz);
        };
        if ((texData.material & 1) || useEnv) {
          for (let u = 0; u < seedU - 1; u++) {
            emit(hline1[u]); emit(hline1[u + 1]); emit(hline2[u]);
            emit(hline1[u + 1]); emit(hline2[u + 1]); emit(hline2[u]);
          }
        }
        mgl.end();
      }

      const tmp = hline1; hline1 = hline2; hline2 = tmp;
    }
  }
}
