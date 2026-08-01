// Minimal fixed-function-style OpenGL 1.x emulation over WebGL2, providing
// just what the Tesla effects need: matrix stacks (projection / modelview /
// texture), immediate mode (QUADS/TRIANGLES/LINES), vertex-array draws,
// blending, depth control, face culling, linear fog and 2D textures.

import { Mat4 } from './mathlib.js';

const MAX_LIGHTS = 8;

// Lighting is per vertex, as OpenGL 1.x does it. The model is narrower
// than the full fixed-function one because of what Energy3D actually
// sets: GL_COLOR_MATERIAL is on with its default GL_AMBIENT_AND_DIFFUSE,
// so glColor drives both material terms; light ambient is left at zero by
// the constructor and never written; material specular is never set, so
// there is no specular term at all. What survives is
//
//     lit = 0.2 (GL's default global ambient) + sum of diffuse * N.L
const VS = `#version 300 es
precision highp float;
in vec3 aPos;
in vec2 aUV;
in vec4 aColor;
in vec3 aNormal;
uniform mat4 uModelView;
uniform mat4 uProjection;
uniform mat4 uTexMatrix;
uniform int uLightCount;
uniform vec4 uLightPos[${MAX_LIGHTS}];      // eye space; w = 0 for directional
uniform vec3 uLightDiffuse[${MAX_LIGHTS}];
uniform vec4 uLightSpot[${MAX_LIGHTS}];     // xyz direction (eye space), w = cos(cutoff) or -1
out vec2 vUV;
out vec4 vColor;
out float vEyeDist;
out vec3 vLit;
void main() {
  vec4 eye = uModelView * vec4(aPos, 1.0);
  gl_Position = uProjection * eye;
  vUV = (uTexMatrix * vec4(aUV, 0.0, 1.0)).xy;
  vColor = aColor;
  vEyeDist = -eye.z;
  vec3 lit = vec3(0.2);
  if (uLightCount > 0) {
    vec3 n = normalize(mat3(uModelView) * aNormal);
    for (int i = 0; i < ${MAX_LIGHTS}; i++) {
      if (i >= uLightCount) break;
      vec3 toLight = uLightPos[i].xyz - eye.xyz * uLightPos[i].w;
      vec3 l = normalize(toLight);
      float d = max(dot(n, l), 0.0);
      if (uLightSpot[i].w > -1.0) {
        float c = dot(normalize(-l), normalize(uLightSpot[i].xyz));
        d *= c < uLightSpot[i].w ? 0.0 : c;
      }
      lit += uLightDiffuse[i] * d;
    }
  }
  vLit = lit;
}`;

const FS = `#version 300 es
precision mediump float;
in vec2 vUV;
in vec4 vColor;
in float vEyeDist;
in vec3 vLit;
uniform sampler2D uSampler;
uniform bool uTexEnabled;
uniform bool uUseVertexColor;
uniform bool uLightingEnabled;
uniform vec4 uColor;
uniform vec3 uFog; // x: enabled, y: start, z: end (linear fog, black)
out vec4 outColor;
void main() {
  // uniform color for array draws: constant vertex attributes are
  // historically unreliable on Safari's Metal-backed WebGL
  vec4 c = uUseVertexColor ? vColor : uColor;
  if (uLightingEnabled) c.rgb *= vLit;
  if (uTexEnabled) c *= texture(uSampler, vUV);
  if (uFog.x > 0.5) {
    float f = clamp((uFog.z - vEyeDist) / (uFog.z - uFog.y), 0.0, 1.0);
    c.rgb *= f;
  }
  outColor = c;
}`;

const MAX_IMM_VERTS = 65536;

export class MiniGL {
  constructor(canvas) {
    const attribs = { alpha: false, antialias: true, depth: true, preserveDrawingBuffer: false };
    let gl = canvas.getContext('webgl2', attribs);
    if (!gl) gl = canvas.getContext('webgl2', { ...attribs, antialias: false });
    if (!gl) throw new Error('WebGL2 not available');
    this.gl = gl;

    this.contextLost = false;
    canvas.addEventListener('webglcontextlost', (e) => {
      e.preventDefault();
      this.contextLost = true;
    });

    // enums
    this.QUADS = 1; this.TRIANGLES = 2; this.LINES = 3;
    this.PROJECTION = 0; this.MODELVIEW = 1; this.TEXTURE = 2;
    this.SRC_ALPHA = gl.SRC_ALPHA;
    this.ONE = gl.ONE;
    this.ONE_MINUS_SRC_ALPHA = gl.ONE_MINUS_SRC_ALPHA;
    this.SRC_COLOR = gl.SRC_COLOR;
    this.FRONT = gl.FRONT;
    this.BACK = gl.BACK;
    this.LEQUAL = gl.LEQUAL;
    this.LESS = gl.LESS;

    // program
    const prog = gl.createProgram();
    const compile = (type, src) => {
      const sh = gl.createShader(type);
      gl.shaderSource(sh, src);
      gl.compileShader(sh);
      if (!gl.getShaderParameter(sh, gl.COMPILE_STATUS)) {
        throw new Error('Shader error: ' + gl.getShaderInfoLog(sh));
      }
      gl.attachShader(prog, sh);
    };
    compile(gl.VERTEX_SHADER, VS);
    compile(gl.FRAGMENT_SHADER, FS);
    gl.linkProgram(prog);
    if (!gl.getProgramParameter(prog, gl.LINK_STATUS)) {
      throw new Error('Link error: ' + gl.getProgramInfoLog(prog));
    }
    gl.useProgram(prog);
    this.prog = prog;
    this.uModelView = gl.getUniformLocation(prog, 'uModelView');
    this.uProjection = gl.getUniformLocation(prog, 'uProjection');
    this.uTexMatrix = gl.getUniformLocation(prog, 'uTexMatrix');
    this.uTexEnabled = gl.getUniformLocation(prog, 'uTexEnabled');
    this.uUseVertexColor = gl.getUniformLocation(prog, 'uUseVertexColor');
    this.uColor = gl.getUniformLocation(prog, 'uColor');
    this.uFog = gl.getUniformLocation(prog, 'uFog');
    this.uLightingEnabled = gl.getUniformLocation(prog, 'uLightingEnabled');
    this.uLightCount = gl.getUniformLocation(prog, 'uLightCount');
    this.uLightPos = gl.getUniformLocation(prog, 'uLightPos');
    this.uLightDiffuse = gl.getUniformLocation(prog, 'uLightDiffuse');
    this.uLightSpot = gl.getUniformLocation(prog, 'uLightSpot');
    this.aPos = gl.getAttribLocation(prog, 'aPos');
    this.aUV = gl.getAttribLocation(prog, 'aUV');
    this.aColor = gl.getAttribLocation(prog, 'aColor');
    this.aNormal = gl.getAttribLocation(prog, 'aNormal');
    this.lightingOn = false;
    this.nLights = 0;

    // matrix stacks
    this.matrices = [new Mat4(), new Mat4(), new Mat4()];
    this.stacks = [[], [], []];
    this.mode = this.MODELVIEW;
    this.matricesDirty = true;

    // render state
    this.curColor = [1, 1, 1, 1];
    this.texEnabled = false;
    this.boundTex = null;
    this.whiteTex = this._makeWhiteTexture();
    this.fogEnabled = false;
    this.fogStart = 0;
    this.fogEnd = 1;

    // immediate mode batch: interleaved pos(3) uv(2) color(4)
    this.immData = new Float32Array(MAX_IMM_VERTS * 9);
    this.immCount = 0;
    this.immMode = 0;
    this.curU = 0; this.curV = 0;

    this.immVBO = gl.createBuffer();
    this.quadIBO = gl.createBuffer();
    this._quadIndexCount = 0;
    this._growQuadIndices(4096);

    // debug: render everything as line skeletons instead of filled triangles
    this.wireframe = false;
    this.quadLineIBO = gl.createBuffer();
    this._quadLineIndexCount = 0;
    this.triLineIBO = gl.createBuffer();
    this._triLineIndexCount = 0;

    // scratch buffers for array draws
    this.posVBO = gl.createBuffer();
    this.uvVBO = gl.createBuffer();
    this.nrmVBO = gl.createBuffer();
    this.idxIBO = gl.createBuffer();

    gl.disable(gl.DEPTH_TEST);
    gl.disable(gl.CULL_FACE);
    gl.enable(gl.BLEND);
    gl.blendFunc(gl.SRC_ALPHA, gl.ONE);
    gl.clearColor(0, 0, 0, 1);
    gl.uniform1i(gl.getUniformLocation(prog, 'uSampler'), 0);
  }

  _makeWhiteTexture() {
    const gl = this.gl;
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, 1, 1, 0, gl.RGBA, gl.UNSIGNED_BYTE,
      new Uint8Array([255, 255, 255, 255]));
    return tex;
  }

  _growQuadIndices(nQuads) {
    if (this._quadIndexCount >= nQuads * 6) return;
    const idx = new Uint32Array(nQuads * 6);
    for (let q = 0; q < nQuads; q++) {
      idx[q * 6 + 0] = q * 4 + 0;
      idx[q * 6 + 1] = q * 4 + 1;
      idx[q * 6 + 2] = q * 4 + 2;
      idx[q * 6 + 3] = q * 4 + 0;
      idx[q * 6 + 4] = q * 4 + 2;
      idx[q * 6 + 5] = q * 4 + 3;
    }
    const gl = this.gl;
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.quadIBO);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, idx, gl.STATIC_DRAW);
    this._quadIndexCount = nQuads * 6;
  }

  _growQuadLineIndices(nQuads) {
    if (this._quadLineIndexCount >= nQuads * 8) return;
    const idx = new Uint32Array(nQuads * 8);
    for (let q = 0; q < nQuads; q++) {
      const b = q * 4;
      idx.set([b, b + 1, b + 1, b + 2, b + 2, b + 3, b + 3, b], q * 8);
    }
    const gl = this.gl;
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.quadLineIBO);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, idx, gl.STATIC_DRAW);
    this._quadLineIndexCount = nQuads * 8;
  }

  _growTriLineIndices(nTris) {
    if (this._triLineIndexCount >= nTris * 6) return;
    const idx = new Uint32Array(nTris * 6);
    for (let t = 0; t < nTris; t++) {
      const b = t * 3;
      idx.set([b, b + 1, b + 1, b + 2, b + 2, b], t * 6);
    }
    const gl = this.gl;
    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.triLineIBO);
    gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, idx, gl.STATIC_DRAW);
    this._triLineIndexCount = nTris * 6;
  }

  // ----- texture management -----

  createTextureFromImage(image, mipmap = false, clamp = false) {
    const gl = this.gl;
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, false);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
    this._setTexParams(mipmap, clamp);
    this.boundTex = tex;
    return tex;
  }

  createTextureFromData(data, width, height, mipmap = false, clamp = false) {
    const gl = this.gl;
    const tex = gl.createTexture();
    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, width, height, 0, gl.RGBA, gl.UNSIGNED_BYTE, data);
    this._setTexParams(mipmap, clamp);
    this.boundTex = tex;
    return tex;
  }

  // `clamp` may be false, true (both axes) or 't' (V only — D3D lets the
  // two addressing modes differ, and Nowhere's credits need exactly that:
  // U carries a scrolling offset that has to wrap, V is a single band)
  _setTexParams(mipmap, clamp = false) {
    const gl = this.gl;
    const wrapS = clamp === true ? gl.CLAMP_TO_EDGE : gl.REPEAT;
    const wrapT = clamp ? gl.CLAMP_TO_EDGE : gl.REPEAT;
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, wrapS);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, wrapT);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    if (mipmap) {
      gl.generateMipmap(gl.TEXTURE_2D);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
    } else {
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    }
  }

  bindTexture(tex) {
    if (this.boundTex !== tex) {
      this.gl.bindTexture(this.gl.TEXTURE_2D, tex);
      this.boundTex = tex;
    }
  }

  // ----- matrix API -----

  matrixMode(m) { this.mode = m; }
  get cur() { return this.matrices[this.mode]; }
  loadIdentity() { this.cur.identity(); this.matricesDirty = true; }
  loadMatrix(mat) { this.cur.copy(mat); this.matricesDirty = true; }
  multMatrix(mat) { this.cur.mult(mat); this.matricesDirty = true; }
  frustum(l, r, b, t, n, f) { this.cur.frustum(l, r, b, t, n, f); this.matricesDirty = true; }
  ortho(l, r, b, t, n, f) { this.cur.ortho(l, r, b, t, n, f); this.matricesDirty = true; }
  translate(x, y, z) { this.cur.translate(x, y, z); this.matricesDirty = true; }
  rotate(a, x, y, z) { this.cur.rotate(a, x, y, z); this.matricesDirty = true; }
  scale(x, y, z) { this.cur.scale(x, y, z); this.matricesDirty = true; }
  pushMatrix() { this.stacks[this.mode].push(this.cur.clone()); }
  popMatrix() {
    const m = this.stacks[this.mode].pop();
    if (m) { this.matrices[this.mode] = m; this.matricesDirty = true; }
  }
  getModelView() { return this.matrices[this.MODELVIEW].clone(); }

  _syncMatrices() {
    if (!this.matricesDirty) return;
    const gl = this.gl;
    gl.uniformMatrix4fv(this.uProjection, false, this.matrices[this.PROJECTION].m);
    gl.uniformMatrix4fv(this.uModelView, false, this.matrices[this.MODELVIEW].m);
    gl.uniformMatrix4fv(this.uTexMatrix, false, this.matrices[this.TEXTURE].m);
    this.matricesDirty = false;
  }

  // ----- render state -----

  color4(r, g, b, a) {
    this.curColor[0] = r; this.curColor[1] = g; this.curColor[2] = b; this.curColor[3] = a;
  }

  enableTexture(on) { this.texEnabled = on; }
  enableBlend(on) { const gl = this.gl; on ? gl.enable(gl.BLEND) : gl.disable(gl.BLEND); }
  enableDepthTest(on) { const gl = this.gl; on ? gl.enable(gl.DEPTH_TEST) : gl.disable(gl.DEPTH_TEST); }
  enableCullFace(on) { const gl = this.gl; on ? gl.enable(gl.CULL_FACE) : gl.disable(gl.CULL_FACE); }
  enableFog(on) { this.fogEnabled = on; }

  // ----- lighting -----
  //
  // `lights` is an array of { pos, diffuse, spotDir, spotCos }, all in eye
  // space, matching what ogl_light::calculate hands to glLightfv. A light
  // with spotCos < 0 is an omni.
  enableLighting(on) { this.lightingOn = !!on; }

  setLights(lights) {
    const gl = this.gl;
    const n = Math.min(lights.length, MAX_LIGHTS);
    const pos = new Float32Array(MAX_LIGHTS * 4);
    const dif = new Float32Array(MAX_LIGHTS * 3);
    const spot = new Float32Array(MAX_LIGHTS * 4);
    for (let i = 0; i < n; i++) {
      const l = lights[i];
      pos.set([l.pos[0], l.pos[1], l.pos[2], l.pos.length > 3 ? l.pos[3] : 1], i * 4);
      dif.set([l.diffuse[0], l.diffuse[1], l.diffuse[2]], i * 3);
      const d = l.spotDir || [0, 0, -1];
      spot.set([d[0], d[1], d[2], l.spotCos === undefined ? -1 : l.spotCos], i * 4);
    }
    this.nLights = n;
    gl.uniform4fv(this.uLightPos, pos);
    gl.uniform3fv(this.uLightDiffuse, dif);
    gl.uniform4fv(this.uLightSpot, spot);
  }
  fog(start, end) { this.fogStart = start; this.fogEnd = end; }
  blendFunc(src, dst) { this.gl.blendFunc(src, dst); }
  depthMask(on) { this.gl.depthMask(!!on); }
  depthFunc(fn) { this.gl.depthFunc(fn); }
  cullFace(face) { this.gl.cullFace(face); }

  clear() {
    const gl = this.gl;
    gl.depthMask(true);
    gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
  }

  _applyCommonUniforms() {
    const gl = this.gl;
    this._syncMatrices();
    gl.uniform1i(this.uTexEnabled, this.texEnabled ? 1 : 0);
    gl.uniform1i(this.uLightingEnabled, this.lightingOn ? 1 : 0);
    gl.uniform1i(this.uLightCount, this.lightingOn ? this.nLights : 0);
    gl.uniform3f(this.uFog, this.fogEnabled ? 1 : 0, this.fogStart, this.fogEnd);
    if (!this.texEnabled) this.bindTexture(this.whiteTex);
  }

  // ----- immediate mode -----

  begin(mode) {
    this.immMode = mode;
    this.immCount = 0;
  }

  texCoord2(u, v) { this.curU = u; this.curV = v; }

  vertex3(x, y, z) {
    const i = this.immCount * 9;
    const d = this.immData;
    d[i] = x; d[i + 1] = y; d[i + 2] = z;
    d[i + 3] = this.curU; d[i + 4] = this.curV;
    d[i + 5] = this.curColor[0]; d[i + 6] = this.curColor[1];
    d[i + 7] = this.curColor[2]; d[i + 8] = this.curColor[3];
    this.immCount++;
  }

  vertex3v(v) { this.vertex3(v.x, v.y, v.z); }

  end() {
    const n = this.immCount;
    if (n === 0) return;
    const gl = this.gl;
    this._applyCommonUniforms();

    gl.bindBuffer(gl.ARRAY_BUFFER, this.immVBO);
    gl.bufferData(gl.ARRAY_BUFFER, this.immData.subarray(0, n * 9), gl.STREAM_DRAW);
    gl.enableVertexAttribArray(this.aPos);
    gl.vertexAttribPointer(this.aPos, 3, gl.FLOAT, false, 36, 0);
    gl.enableVertexAttribArray(this.aUV);
    gl.vertexAttribPointer(this.aUV, 2, gl.FLOAT, false, 36, 12);
    gl.enableVertexAttribArray(this.aColor);
    gl.vertexAttribPointer(this.aColor, 4, gl.FLOAT, false, 36, 20);
    gl.uniform1i(this.uUseVertexColor, 1);
    // The immediate-mode buffer has no normals; without this, aNormal stays
    // enabled and pointing into nrmVBO from whatever mesh drew last.
    if (this.aNormal >= 0) {
      gl.disableVertexAttribArray(this.aNormal);
      gl.vertexAttrib3f(this.aNormal, 0, 0, 1);
    }

    if (this.immMode === this.QUADS) {
      const quads = n >> 2;
      if (this.wireframe) {
        this._growQuadLineIndices(quads);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.quadLineIBO);
        gl.drawElements(gl.LINES, quads * 8, gl.UNSIGNED_INT, 0);
      } else {
        this._growQuadIndices(quads);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.quadIBO);
        gl.drawElements(gl.TRIANGLES, quads * 6, gl.UNSIGNED_INT, 0);
      }
    } else if (this.immMode === this.TRIANGLES) {
      if (this.wireframe) {
        const tris = Math.floor(n / 3);
        this._growTriLineIndices(tris);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.triLineIBO);
        gl.drawElements(gl.LINES, tris * 6, gl.UNSIGNED_INT, 0);
      } else {
        gl.drawArrays(gl.TRIANGLES, 0, n);
      }
    } else if (this.immMode === this.LINES) {
      gl.drawArrays(gl.LINES, 0, n);
    }
    this.immCount = 0;
  }

  // ----- vertex-array draws (glVertexPointer/glTexCoordPointer/glDrawElements) -----
  // positions: Float32Array of xyz, uvs: Float32Array of uv, indices: Uint32Array
  // colors (optional): Float32Array of rgba per vertex — the D3D-era
  // XYZ|DIFFUSE|TEX1 vertex layout; omitted, the current color applies.

  // An unindexed triangle list, for effects that expand their own indices
  // — xotrack's do, because they emit through glBegin/glVertex3f and
  // recompute each vertex as they go.
  drawArraysTri(positions, uvs) {
    const gl = this.gl;
    this._applyCommonUniforms();

    gl.bindBuffer(gl.ARRAY_BUFFER, this.posVBO);
    gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STREAM_DRAW);
    gl.enableVertexAttribArray(this.aPos);
    gl.vertexAttribPointer(this.aPos, 3, gl.FLOAT, false, 0, 0);

    gl.bindBuffer(gl.ARRAY_BUFFER, this.uvVBO);
    gl.bufferData(gl.ARRAY_BUFFER, uvs, gl.STREAM_DRAW);
    gl.enableVertexAttribArray(this.aUV);
    gl.vertexAttribPointer(this.aUV, 2, gl.FLOAT, false, 0, 0);

    gl.disableVertexAttribArray(this.aColor);
    gl.uniform1i(this.uUseVertexColor, 0);
    gl.uniform4fv(this.uColor, this.curColor);
    if (this.aNormal >= 0) {
      gl.disableVertexAttribArray(this.aNormal);
      gl.vertexAttrib3f(this.aNormal, 0, 0, 1);
    }

    gl.drawArrays(gl.TRIANGLES, 0, positions.length / 3);
  }

  // An unindexed line list, for showCylinder — which emits its wireframe
  // through glBegin(GL_LINES) a triangle at a time.
  drawArraysLines(positions) {
    const gl = this.gl;
    this._applyCommonUniforms();
    gl.bindBuffer(gl.ARRAY_BUFFER, this.posVBO);
    gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STREAM_DRAW);
    gl.enableVertexAttribArray(this.aPos);
    gl.vertexAttribPointer(this.aPos, 3, gl.FLOAT, false, 0, 0);
    gl.disableVertexAttribArray(this.aUV);
    gl.vertexAttrib2f(this.aUV, 0, 0);
    gl.disableVertexAttribArray(this.aColor);
    gl.uniform1i(this.uUseVertexColor, 0);
    gl.uniform4fv(this.uColor, this.curColor);
    if (this.aNormal >= 0) {
      gl.disableVertexAttribArray(this.aNormal);
      gl.vertexAttrib3f(this.aNormal, 0, 0, 1);
    }
    gl.drawArrays(gl.LINES, 0, positions.length / 3);
  }

  drawElements(positions, uvs, indices, colors = null, normals = null, mode = null) {
    const gl = this.gl;
    this._applyCommonUniforms();

    gl.bindBuffer(gl.ARRAY_BUFFER, this.posVBO);
    gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STREAM_DRAW);
    gl.enableVertexAttribArray(this.aPos);
    gl.vertexAttribPointer(this.aPos, 3, gl.FLOAT, false, 0, 0);

    if (normals && this.aNormal >= 0) {
      gl.bindBuffer(gl.ARRAY_BUFFER, this.nrmVBO);
      gl.bufferData(gl.ARRAY_BUFFER, normals, gl.STREAM_DRAW);
      gl.enableVertexAttribArray(this.aNormal);
      gl.vertexAttribPointer(this.aNormal, 3, gl.FLOAT, false, 0, 0);
    } else if (this.aNormal >= 0) {
      gl.disableVertexAttribArray(this.aNormal);
      gl.vertexAttrib3f(this.aNormal, 0, 0, 1);
    }

    gl.bindBuffer(gl.ARRAY_BUFFER, this.uvVBO);
    gl.bufferData(gl.ARRAY_BUFFER, uvs, gl.STREAM_DRAW);
    gl.enableVertexAttribArray(this.aUV);
    gl.vertexAttribPointer(this.aUV, 2, gl.FLOAT, false, 0, 0);

    if (colors) {
      if (!this.colVBO) this.colVBO = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, this.colVBO);
      gl.bufferData(gl.ARRAY_BUFFER, colors, gl.STREAM_DRAW);
      gl.enableVertexAttribArray(this.aColor);
      gl.vertexAttribPointer(this.aColor, 4, gl.FLOAT, false, 0, 0);
      gl.uniform1i(this.uUseVertexColor, 1);
    } else {
      gl.disableVertexAttribArray(this.aColor);
      gl.uniform1i(this.uUseVertexColor, 0);
      gl.uniform4fv(this.uColor, this.curColor);
    }

    gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.idxIBO);
    if (this.wireframe) {
      const tris = Math.floor(indices.length / 3);
      const lines = new Uint32Array(tris * 6);
      for (let t = 0; t < tris; t++) {
        const a = indices[t * 3], b = indices[t * 3 + 1], c = indices[t * 3 + 2];
        lines.set([a, b, b, c, c, a], t * 6);
      }
      gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, lines, gl.STREAM_DRAW);
      gl.drawElements(gl.LINES, lines.length, gl.UNSIGNED_INT, 0);
    } else {
      gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, indices, gl.STREAM_DRAW);
      gl.drawElements(mode === null ? gl.TRIANGLES : mode, indices.length, gl.UNSIGNED_INT, 0);
    }
  }
}
