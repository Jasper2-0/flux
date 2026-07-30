// Parser for Contour's "ARSE" scene format (Tim's scene replayer data).
// Layout documented in the repository's reverse-engineering notes:
// header, frame-major per-object animation transforms, per-object meshes,
// and a fixed-capacity trailing camera track (896 records; first `frames`
// records meaningful).

export function parseARSE(bytes) {
  const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let p = 0;
  const u32 = () => { const v = dv.getUint32(p, true); p += 4; return v; };
  const f32 = () => { const v = dv.getFloat32(p, true); p += 4; return v; };

  const magic = String.fromCharCode(bytes[0], bytes[1], bytes[2], bytes[3]);
  p = 4;
  if (magic !== 'ARSE') throw new Error('not an ARSE scene: ' + magic);

  const frames = u32();
  const fps = u32();
  const nObjects = u32();
  const flags = u32();
  const zfar = f32();
  const znear = f32();
  const fov = f32();

  // animation: frames * objects records of 3x3 rotation + translation
  const anim = [];
  for (let f = 0; f < frames; f++) {
    const perObject = [];
    for (let o = 0; o < nObjects; o++) {
      const m = new Float32Array(9);
      for (let i = 0; i < 9; i++) m[i] = f32();
      const t = new Float32Array(3);
      for (let i = 0; i < 3; i++) t[i] = f32();
      perObject.push({ m, t });
    }
    anim.push(perObject);
  }

  // geometry
  const objects = [];
  for (let o = 0; o < nObjects; o++) {
    const nVerts = u32();
    const nFaces = u32();
    const unknown = [u32(), u32(), u32()];
    const positions = new Float32Array(nVerts * 3);
    const normals = new Float32Array(nVerts * 3);
    const uv = new Float32Array(nVerts * 2);
    for (let v = 0; v < nVerts; v++) {
      positions[v * 3] = f32(); positions[v * 3 + 1] = f32(); positions[v * 3 + 2] = f32();
      normals[v * 3] = f32(); normals[v * 3 + 1] = f32(); normals[v * 3 + 2] = f32();
      uv[v * 2] = f32(); uv[v * 2 + 1] = f32();
    }
    const faces = new Uint32Array(nFaces * 3);
    for (let i = 0; i < nFaces * 3; i++) {
      faces[i] = dv.getUint16(p, true); p += 2;
    }
    objects.push({ nVerts, nFaces, unknown, positions, normals, uv, faces });
  }

  // trailing camera track: fixed 896-record capacity, first `frames` valid
  const camera = [];
  for (let f = 0; f < frames; f++) {
    const m = new Float32Array(9);
    for (let i = 0; i < 9; i++) m[i] = f32();
    const t = new Float32Array(3);
    for (let i = 0; i < 3; i++) t[i] = f32();
    camera.push({ m, t });
  }

  return { frames, fps, flags, zfar, znear, fov, anim, objects, camera };
}
