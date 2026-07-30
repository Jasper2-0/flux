// 3DS mesh parser for the Tesla WebGL port.
//
// Replicates the semantics of the original loader the demo linked against:
//  - vertices are read with Y and Z swapped (3DS is Z-up, demo is Y-up)
//  - per-face-corner UVs come from the UV table with v flipped (1 - v);
//    the effects collapse them to per-vertex anyway, so we store per-vertex
//  - vertices not covered by a UV table get spherical coords derived from the
//    vertex normal: u = nx*0.5+0.5, v = ny*0.5+0.5
//  - vertex normals ("wglobal") are the average of adjacent face normals
//  - the MATRIX chunk (read with the same axis swap, then inverted) maps the
//    mesh into local/pivot space: vlocal = inv(xform) * vglobal, and
//    wlocal = rotate-only inv(xform) * wglobal

const C_MAIN = 0x4d4d;
const C_MESH = 0x3d3d;
const ELEMENT_NAME = 0x4000;
const OBJECT = 0x4100;
const VERTICES_TAB = 0x4110;
const FACES_TAB = 0x4120;
const UV_TAB = 0x4140;
const MATRIX = 0x4160;

export function parse3DS(bytes) {
  const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const objects = [];

  function walk(start, end, handler) {
    let p = start;
    while (p + 6 <= end) {
      const id = dv.getUint16(p, true);
      const len = dv.getUint32(p + 2, true);
      if (len < 6 || p + len > end) break;
      handler(id, p + 6, p + len);
      p += len;
    }
  }

  walk(0, bytes.byteLength, (id, body, bodyEnd) => {
    if (id !== C_MAIN) return;
    walk(body, bodyEnd, (id2, b2, e2) => {
      if (id2 !== C_MESH) return;
      walk(b2, e2, (id3, b3, e3) => {
        if (id3 !== ELEMENT_NAME) return;
        // NUL-terminated name, then sub-chunks
        let p = b3;
        let name = '';
        while (p < e3 && bytes[p] !== 0) { name += String.fromCharCode(bytes[p]); p++; }
        p++;
        walk(p, e3, (id4, b4, e4) => {
          if (id4 !== OBJECT) return;
          objects.push(readObject(name, b4, e4));
        });
      });
    });
  });

  function readObject(name, start, end) {
    const obj = { name, nvertices: 0, nfaces: 0 };
    let vglobal = null;       // Float32Array xyz (y/z swapped)
    let faces = null;         // Uint32Array, 3 indices per face
    let uvRaw = null;         // Float32Array uv from file (per original vertex)
    let hasUV = null;         // Uint8Array flags: 1 if vertex got UV from table
    let xform = null;         // 4x3 rows [ [a,b,c], [d,e,f], [g,h,i], [j,k,l] ]

    walk(start, end, (id, b, e) => {
      if (id === VERTICES_TAB) {
        const n = dv.getUint16(b, true);
        vglobal = new Float32Array(n * 3);
        let p = b + 2;
        for (let i = 0; i < n; i++) {
          vglobal[i * 3] = dv.getFloat32(p, true);
          vglobal[i * 3 + 1] = dv.getFloat32(p + 8, true);
          vglobal[i * 3 + 2] = dv.getFloat32(p + 4, true);
          p += 12;
        }
        obj.nvertices = n;
      } else if (id === FACES_TAB) {
        const n = dv.getUint16(b, true);
        faces = new Uint32Array(n * 3);
        let p = b + 2;
        for (let i = 0; i < n; i++) {
          faces[i * 3] = dv.getUint16(p, true);
          faces[i * 3 + 1] = dv.getUint16(p + 2, true);
          faces[i * 3 + 2] = dv.getUint16(p + 4, true);
          p += 8; // 3 indices + flags
        }
        obj.nfaces = n;
      } else if (id === UV_TAB) {
        const n = dv.getUint16(b, true);
        uvRaw = new Float32Array(n * 2);
        let p = b + 2;
        for (let i = 0; i < n; i++) {
          uvRaw[i * 2] = dv.getFloat32(p, true);
          uvRaw[i * 2 + 1] = dv.getFloat32(p + 4, true);
          p += 8;
        }
      } else if (id === MATRIX) {
        // Same element shuffle as the original ReadMatrix (y/z axis swap on
        // both rows and columns), then inverted below.
        const f = [];
        for (let i = 0; i < 12; i++) f.push(dv.getFloat32(b + i * 4, true));
        xform = [
          [f[0], f[2], f[1]],
          [f[6], f[8], f[7]],
          [f[3], f[5], f[4]],
          [f[9], f[11], f[10]],
        ];
        invert43(xform);
      }
    });

    const n = obj.nvertices;

    // face normals -> averaged vertex normals (wglobal)
    const wglobal = new Float32Array(n * 3);
    if (faces && vglobal) {
      const counts = new Uint32Array(n);
      for (let i = 0; i < obj.nfaces; i++) {
        const a = faces[i * 3], bIdx = faces[i * 3 + 1], c = faces[i * 3 + 2];
        const ax = vglobal[a * 3], ay = vglobal[a * 3 + 1], az = vglobal[a * 3 + 2];
        let e1x = vglobal[bIdx * 3] - ax, e1y = vglobal[bIdx * 3 + 1] - ay, e1z = vglobal[bIdx * 3 + 2] - az;
        let e2x = vglobal[c * 3] - ax, e2y = vglobal[c * 3 + 1] - ay, e2z = vglobal[c * 3 + 2] - az;
        let nx = e1y * e2z - e1z * e2y;
        let ny = e1z * e2x - e1x * e2z;
        let nz = e1x * e2y - e1y * e2x;
        const len = Math.sqrt(nx * nx + ny * ny + nz * nz) || 1;
        nx /= len; ny /= len; nz /= len;
        for (const v of [a, bIdx, c]) {
          wglobal[v * 3] += nx; wglobal[v * 3 + 1] += ny; wglobal[v * 3 + 2] += nz;
          counts[v]++;
        }
      }
      for (let v = 0; v < n; v++) {
        const cnt = counts[v] || 1;
        wglobal[v * 3] /= cnt; wglobal[v * 3 + 1] /= cnt; wglobal[v * 3 + 2] /= cnt;
      }
    }

    // per-vertex UV: from the UV table (v flipped) or spherical from normal
    const uv = new Float32Array(n * 2);
    for (let v = 0; v < n; v++) {
      if (uvRaw && v * 2 + 1 < uvRaw.length) {
        uv[v * 2] = uvRaw[v * 2];
        uv[v * 2 + 1] = 1 - uvRaw[v * 2 + 1];
      } else {
        uv[v * 2] = wglobal[v * 3] * 0.5 + 0.5;
        uv[v * 2 + 1] = wglobal[v * 3 + 1] * 0.5 + 0.5;
      }
    }

    // vlocal / wlocal via the inverted MATRIX chunk
    const vlocal = new Float32Array(n * 3);
    const wlocal = new Float32Array(n * 3);
    for (let v = 0; v < n; v++) {
      const x = vglobal[v * 3], y = vglobal[v * 3 + 1], z = vglobal[v * 3 + 2];
      const wx = wglobal[v * 3], wy = wglobal[v * 3 + 1], wz = wglobal[v * 3 + 2];
      if (xform) {
        const m = xform;
        vlocal[v * 3] = x * m[0][0] + y * m[1][0] + z * m[2][0] + m[3][0];
        vlocal[v * 3 + 1] = x * m[0][1] + y * m[1][1] + z * m[2][1] + m[3][1];
        vlocal[v * 3 + 2] = x * m[0][2] + y * m[1][2] + z * m[2][2] + m[3][2];
        wlocal[v * 3] = wx * m[0][0] + wy * m[1][0] + wz * m[2][0];
        wlocal[v * 3 + 1] = wx * m[0][1] + wy * m[1][1] + wz * m[2][1];
        wlocal[v * 3 + 2] = wx * m[0][2] + wy * m[1][2] + wz * m[2][2];
      } else {
        vlocal[v * 3] = x; vlocal[v * 3 + 1] = y; vlocal[v * 3 + 2] = z;
        wlocal[v * 3] = wx; wlocal[v * 3 + 1] = wy; wlocal[v * 3 + 2] = wz;
      }
    }

    obj.vglobal = vglobal;
    obj.vlocal = vlocal;
    obj.wlocal = wlocal;
    obj.wglobal = wglobal;
    obj.uv = uv;
    obj.faces = faces;
    return obj;
  }

  // In-place inverse of a 4x3 affine matrix stored as rows, ported from the
  // original mtrxInvert.
  function invert43(m) {
    const a = m[0][0], b = m[0][1], c = m[0][2];
    const d = m[1][0], e = m[1][1], f = m[1][2];
    const g = m[2][0], h = m[2][1], i = m[2][2];
    const j = m[3][0], k = m[3][1], l = m[3][2];
    const w = 1.0 / (a * (e * i - f * h) - (b * (d * i - f * g) + c * (e * g - d * h)));
    m[0][0] = (e * i - f * h) * w; m[0][1] = (c * h - b * i) * w; m[0][2] = (b * f - c * e) * w;
    m[1][0] = (f * g - d * i) * w; m[1][1] = (a * i - c * g) * w; m[1][2] = (c * d - a * f) * w;
    m[2][0] = (d * h - e * g) * w; m[2][1] = (b * g - a * h) * w; m[2][2] = (a * e - b * d) * w;
    m[3][0] = (e * (g * l - i * j) + f * (h * j - g * k) - d * (h * l - i * k)) * w;
    m[3][1] = (a * (h * l - i * k) + b * (i * j - g * l) + c * (g * k - h * j)) * w;
    m[3][2] = (b * (d * l - f * j) + c * (e * j - d * k) - a * (e * l - f * k)) * w;
  }

  return { objects };
}
