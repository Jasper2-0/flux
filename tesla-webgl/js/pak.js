// data.pak archive reader + image decoding for the Tesla WebGL port.
//
// Pak layout: raw file blobs, then a directory, then a u32 tail pointer to the
// directory. Directory: u32 count, then per entry u32 offset, u32 size,
// u32 reserved, u32 nameLen, and a nameLen-byte NUL-terminated DOS-style path
// (e.g. "DATA\\TEXTURES\\SPHERE.JPG").

export class Pak {
  constructor(buffer) {
    this.bytes = new Uint8Array(buffer);
    this.entries = new Map();
    const dv = new DataView(buffer);
    const dirOffset = dv.getUint32(buffer.byteLength - 4, true);
    let p = dirOffset;
    const count = dv.getUint32(p, true); p += 4;
    for (let i = 0; i < count; i++) {
      const offset = dv.getUint32(p, true); p += 4;
      const size = dv.getUint32(p, true); p += 4;
      p += 4; // reserved
      const nameLen = dv.getUint32(p, true); p += 4;
      let name = '';
      for (let c = 0; c < nameLen; c++) {
        const ch = this.bytes[p + c];
        if (ch === 0) break;
        name += String.fromCharCode(ch);
      }
      p += nameLen;
      this.entries.set(Pak.normalize(name), { offset, size });
    }
  }

  // "data/textures/sphere.jpg" -> "DATA/TEXTURES/SPHERE.JPG"
  static normalize(name) {
    return name.replace(/\\/g, '/').toUpperCase();
  }

  get(name) {
    const e = this.entries.get(Pak.normalize(name));
    if (!e) throw new Error('pak: file not found: ' + name);
    return this.bytes.subarray(e.offset, e.offset + e.size);
  }
}

// Decode an uncompressed TGA (type 2) to RGBA, flipping to top-down row order
// and converting BGR(A) to RGBA — the same as the original ImageLib loader.
export function decodeTGA(bytes) {
  const dv = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const idLen = bytes[0];
  const imageType = bytes[2];
  if (imageType !== 2) throw new Error('tga: unsupported image type ' + imageType);
  const width = dv.getUint16(12, true);
  const height = dv.getUint16(14, true);
  const bpp = bytes[16];
  const nBytes = bpp >> 3;
  let src = 18 + idLen;
  const out = new Uint8Array(width * height * 4);
  for (let y = 0; y < height; y++) {
    const dstRow = (height - y - 1) * width * 4;
    for (let x = 0; x < width; x++) {
      const s = src + (y * width + x) * nBytes;
      const d = dstRow + x * 4;
      out[d] = bytes[s + 2];
      out[d + 1] = bytes[s + 1];
      out[d + 2] = bytes[s];
      out[d + 3] = nBytes === 4 ? bytes[s + 3] : 255;
    }
  }
  return { width, height, data: out };
}

const MIME = { JPG: 'image/jpeg', PNG: 'image/png' };

// Texture manager: mirrors CTextureManager but async. All textures are
// preloaded before the demo starts; loadTexture() then returns synchronously.
export class TexManager {
  constructor(mgl, pak) {
    this.mgl = mgl;
    this.pak = pak;
    this.cache = new Map();
  }

  async preload(name, mipmap = false) {
    const key = Pak.normalize(name) + (mipmap ? '#mip' : '');
    if (this.cache.has(key)) return this.cache.get(key);
    const bytes = this.pak.get(name);
    const ext = name.slice(name.lastIndexOf('.') + 1).toUpperCase();
    let tex;
    if (ext === 'TGA') {
      const img = decodeTGA(bytes);
      tex = this.mgl.createTextureFromData(img.data, img.width, img.height, mipmap);
    } else {
      const blob = new Blob([bytes], { type: MIME[ext] || 'application/octet-stream' });
      const bitmap = await createImageBitmap(blob);
      tex = this.mgl.createTextureFromImage(bitmap, mipmap);
      bitmap.close();
    }
    this.cache.set(key, tex);
    return tex;
  }

  loadTexture(name, mipmap = false) {
    const key = Pak.normalize(name) + (mipmap ? '#mip' : '');
    const tex = this.cache.get(key);
    if (!tex) throw new Error('texture not preloaded: ' + name);
    return tex;
  }
}
