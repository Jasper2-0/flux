// Interactive layer of the Tesla port write-up.
// Expects globals: G (bundled engine namespace) and PAK_B64 (data.pak).

(async () => {
  const $ = (sel) => document.querySelector(sel);
  const gateStatus = $('#gate-status');
  const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches;

  const setGate = (msg) => { if (gateStatus) gateStatus.textContent = msg; };

  function b64ToBuffer(b64) {
    const bin = atob(b64);
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    return bytes.buffer;
  }

  function fmtTime(sec) {
    const m = Math.floor(sec / 60);
    const s = sec - m * 60;
    return m + ':' + (s < 10 ? '0' : '') + s.toFixed(0).padStart(2, '0');
  }

  // ---------- palette ----------
  const EFFECT_COLORS = {
    spinzoom: '#6fc3ff', shadeball: '#8ad0c8', splines: '#f0e28a',
    ffdenv: '#b48cff', bands: '#7d9cff', energystream: '#f0b46a',
    tubes: '#8ae0a0', polkalike: '#ff9bb0', tree: '#c0f07a',
    facemorph: '#e0a0ff', thing: '#a0b8c8',
  };

  // ---------- boot: pak + engine ----------
  setGate('decoding data.pak…');
  const pak = new G.Pak(b64ToBuffer(PAK_B64));

  // ---------- hero timeline ----------
  const TL = [
    { key: 'spinzoom', label: 'SPINZOOM', s: 0, e: 24.5, lane: 0, href: '#e-spinzoom' },
    { key: 'shadeball', label: 'SHADEBALL', s: 24.5, e: 48.5, lane: 0, href: '#e-shadeball' },
    { key: 'splines', label: 'SPLINES', s: 48.5, e: 67.7, lane: 0, href: '#e-splines' },
    { key: 'ffdenv', label: 'FFDENV', s: 67.7, e: 87, lane: 0, href: '#e-ffdenv' },
    { key: 'bands', label: 'BANDS', s: 69, e: 85, lane: 1, href: '#e-ffdenv' },
    { key: 'energystream', label: 'ENERGYSTREAM', s: 87, e: 145, lane: 0, href: '#e-energystream' },
    { key: 'tubes', label: 'TUBES', s: 145, e: 165, lane: 0, href: '#e-tubes' },
    { key: 'polkalike', label: 'POLKALIKE', s: 165, e: 203, lane: 0, href: '#e-polkalike' },
    { key: 'tree', label: 'TREE', s: 186, e: 201, lane: 1, href: '#e-polkalike' },
    { key: 'facemorph', label: 'FACEMORPH', s: 203, e: 222.5, lane: 0, href: '#e-facemorph' },
    { key: 'thing', label: 'THING', s: 222.5, e: 254, lane: 0, href: '#e-thing' },
  ];

  (function buildTimeline() {
    const svg = $('#timeline');
    if (!svg) return;
    const W = 960, H = 128, L = 8, R = 8;
    const x = (t) => L + (t / 254) * (W - L - R);
    svg.setAttribute('viewBox', '0 0 ' + W + ' ' + H);
    const NS = 'http://www.w3.org/2000/svg';
    let out = '';
    for (let t = 0; t <= 254; t += 30) {
      out += '<rect class="tick" x="' + x(t).toFixed(1) + '" y="26" width="1" height="74"></rect>' +
        '<text x="' + x(t).toFixed(1) + '" y="118" font-size="9" fill="#67737f">' + fmtTime(t) + '</text>';
    }
    svg.innerHTML = out;
    for (const b of TL) {
      const g = document.createElementNS(NS, 'g');
      g.setAttribute('class', 'blk');
      g.setAttribute('tabindex', '0');
      g.setAttribute('role', 'link');
      const y = b.lane === 0 ? 30 : 72, h = b.lane === 0 ? 38 : 24;
      const rect = document.createElementNS(NS, 'rect');
      rect.setAttribute('x', x(b.s).toFixed(1));
      rect.setAttribute('y', y);
      rect.setAttribute('width', (x(b.e) - x(b.s)).toFixed(1));
      rect.setAttribute('height', h);
      rect.setAttribute('rx', '3');
      rect.setAttribute('fill', EFFECT_COLORS[b.key]);
      rect.setAttribute('opacity', '0.55');
      g.appendChild(rect);
      const label = document.createElementNS(NS, 'text');
      label.setAttribute('x', (x(b.s) + 5).toFixed(1));
      label.setAttribute('y', y + (b.lane === 0 ? 16 : 15));
      label.setAttribute('font-size', b.lane === 0 ? '9.5' : '8.5');
      label.setAttribute('fill', '#06080a');
      label.setAttribute('font-weight', 'bold');
      label.textContent = b.label;
      if (x(b.e) - x(b.s) > b.label.length * 6.5) g.appendChild(label);
      const go = () => document.querySelector(b.href) &&
        document.querySelector(b.href).scrollIntoView({ behavior: reducedMotion ? 'auto' : 'smooth' });
      g.addEventListener('click', go);
      g.addEventListener('keydown', (e) => { if (e.key === 'Enter') go(); });
      svg.appendChild(g);
    }
  })();

  // ---------- asset browser ----------
  setGate('decoding textures…');
  const assetGrid = $('#assets');
  const pakTable = $('#paktable');

  function blobToDataURL(blob) {
    return new Promise((res, rej) => {
      const r = new FileReader();
      r.onload = () => res(r.result);
      r.onerror = () => rej(r.error);
      r.readAsDataURL(blob);
    });
  }

  if (pakTable) {
    let rows = '<tr><th>path in pak</th><th>offset</th><th>size</th></tr>';
    for (const [name, e] of pak.entries) {
      rows += '<tr><td>' + name + '</td><td class="num">0x' +
        e.offset.toString(16).padStart(6, '0') + '</td><td class="num">' +
        (e.size / 1024).toFixed(1) + ' KB</td></tr>';
    }
    pakTable.innerHTML = rows;
  }

  if (assetGrid) {
    for (const [name, entry] of pak.entries) {
      const ext = name.slice(name.lastIndexOf('.') + 1);
      if (ext !== 'JPG' && ext !== 'PNG' && ext !== 'TGA') continue;
      const bytes = pak.bytes.subarray(entry.offset, entry.offset + entry.size);
      const fig = document.createElement('figure');
      let dims = '';
      if (ext === 'TGA') {
        const img = G.decodeTGA(bytes);
        const cv = document.createElement('canvas');
        cv.width = img.width; cv.height = img.height;
        cv.getContext('2d').putImageData(new ImageData(
          new Uint8ClampedArray(img.data.buffer), img.width, img.height), 0, 0);
        fig.appendChild(cv);
        dims = img.width + '×' + img.height;
        finishFig();
      } else {
        const el = document.createElement('img');
        el.alt = name;
        blobToDataURL(new Blob([bytes], { type: ext === 'JPG' ? 'image/jpeg' : 'image/png' }))
          .then((url) => {
            el.src = url;
            el.onload = () => {
              dims = el.naturalWidth + '×' + el.naturalHeight;
              finishFig();
            };
          });
        fig.appendChild(el);
      }
      function finishFig() {
        const cap = fig.querySelector('figcaption') || document.createElement('figcaption');
        cap.textContent = name.replace('DATA/TEXTURES/', '').replace('DATA/3D/', '') +
          ' · ' + dims + ' · ' + (entry.size / 1024).toFixed(0) + ' KB';
        fig.appendChild(cap);
      }
      fig.addEventListener('click', () => fig.classList.toggle('big'));
      assetGrid.appendChild(fig);
    }
  }

  // ---------- mesh lab ----------
  setGate('parsing meshes…');
  const scenes = {
    meta: G.parse3DS(pak.get('data/3d/meta.3ds')),
    faces: G.parse3DS(pak.get('data/3d/faces.3ds')),
    kbuu: G.parse3DS(pak.get('data/3d/kbuu2.3ds')),
  };
  // which vertex set each effect actually reads
  const MESH_SPACE = { meta: 'vlocal', faces: 'vglobal', kbuu: 'vlocal' };

  (function meshLab() {
    const canvas = $('#meshcanvas');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const selScene = $('#mesh-scene'), selA = $('#mesh-a'), selB = $('#mesh-b');
    const slider = $('#mesh-t'), info = $('#mesh-info');
    const edgeCache = new Map();
    let angle = 0.6, visible = false, dragging = false, lastX = 0;

    function edges(sceneKey, objIdx) {
      const id = sceneKey + ':' + objIdx;
      if (edgeCache.has(id)) return edgeCache.get(id);
      const obj = scenes[sceneKey].objects[objIdx];
      const set = new Set();
      const f = obj.faces;
      for (let i = 0; i < f.length; i += 3) {
        for (const [a, b] of [[f[i], f[i + 1]], [f[i + 1], f[i + 2]], [f[i + 2], f[i]]]) {
          set.add(a < b ? a * 65536 + b : b * 65536 + a);
        }
      }
      const arr = new Uint32Array(set.size * 2);
      let k = 0;
      for (const e of set) { arr[k++] = Math.floor(e / 65536); arr[k++] = e % 65536; }
      edgeCache.set(id, arr);
      return arr;
    }

    function populate() {
      const sceneKey = selScene.value;
      const n = scenes[sceneKey].objects.length;
      for (const sel of [selA, selB]) {
        sel.innerHTML = '';
        for (let i = 0; i < n; i++) {
          const o = document.createElement('option');
          o.value = i;
          o.textContent = scenes[sceneKey].objects[i].name || ('obj ' + i);
          sel.appendChild(o);
        }
      }
      selA.value = 0;
      selB.value = Math.min(1, n - 1);
    }

    function draw() {
      const sceneKey = selScene.value;
      const scene = scenes[sceneKey];
      const space = MESH_SPACE[sceneKey];
      const a = scene.objects[+selA.value], b = scene.objects[+selB.value];
      const va = a[space], vb = b[space];
      const morphOK = a.nvertices === b.nvertices;
      let t = morphOK ? +slider.value : 0;
      t = Math.cos((1 - t) * Math.PI) * 0.5 + 0.5; // the demo's cosine ease

      const W = canvas.width, H = canvas.height;
      ctx.clearRect(0, 0, W, H);

      // fit: bbox of object A
      let minX = 1e9, maxX = -1e9, minY = 1e9, maxY = -1e9, minZ = 1e9, maxZ = -1e9;
      for (let i = 0; i < va.length; i += 3) {
        minX = Math.min(minX, va[i]); maxX = Math.max(maxX, va[i]);
        minY = Math.min(minY, va[i + 1]); maxY = Math.max(maxY, va[i + 1]);
        minZ = Math.min(minZ, va[i + 2]); maxZ = Math.max(maxZ, va[i + 2]);
      }
      const cx = (minX + maxX) / 2, cy = (minY + maxY) / 2, cz = (minZ + maxZ) / 2;
      const ext = Math.max(maxX - minX, maxY - minY, maxZ - minZ) || 1;
      const scale = Math.min(W, H) * 0.72 / ext;

      const cosA = Math.cos(angle), sinA = Math.sin(angle);
      const tilt = 0.32, cosT = Math.cos(tilt), sinT = Math.sin(tilt);
      const n = a.nvertices;
      const px = new Float32Array(n), py = new Float32Array(n);
      for (let i = 0; i < n; i++) {
        let x = va[i * 3], y = va[i * 3 + 1], z = va[i * 3 + 2];
        if (morphOK && t > 0) {
          x += (vb[i * 3] - x) * t;
          y += (vb[i * 3 + 1] - y) * t;
          z += (vb[i * 3 + 2] - z) * t;
        }
        x -= cx; y -= cy; z -= cz;
        const rx = x * cosA + z * sinA;
        const rz = -x * sinA + z * cosA;
        const ry = y * cosT - rz * sinT;
        px[i] = W / 2 + rx * scale;
        py[i] = H / 2 - ry * scale;
      }

      ctx.strokeStyle = 'rgba(111,195,255,0.34)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      const eArr = edges(sceneKey, +selA.value);
      for (let i = 0; i < eArr.length; i += 2) {
        ctx.moveTo(px[eArr[i]], py[eArr[i]]);
        ctx.lineTo(px[eArr[i + 1]], py[eArr[i + 1]]);
      }
      ctx.stroke();

      info.textContent = a.nvertices + ' verts · ' + (a.faces.length / 3) + ' faces' +
        (morphOK ? '' : ' · morph n/a (vertex counts differ)');
    }

    function loop() {
      if (visible) {
        if (!reducedMotion && !dragging) angle += 0.004;
        draw();
      }
      requestAnimationFrame(loop);
    }

    new IntersectionObserver((es) => { visible = es[0].isIntersecting; }).observe(canvas);
    canvas.addEventListener('pointerdown', (e) => { dragging = true; lastX = e.clientX; });
    window.addEventListener('pointermove', (e) => {
      if (dragging) { angle += (e.clientX - lastX) * 0.008; lastX = e.clientX; }
    });
    window.addEventListener('pointerup', () => { dragging = false; });
    selScene.addEventListener('change', () => { populate(); });
    populate();
    selScene.value = 'faces';
    populate();
    selB.value = Math.min(1, scenes.faces.objects.length - 1);
    loop();
  })();

  // ---------- live GL viewers ----------
  setGate('uploading textures to WebGL…');

  const glCanvas = document.createElement('canvas');
  glCanvas.width = 640;
  glCanvas.height = 480;
  let mgl = null, tex = null, glFailed = null;
  const counters = { draws: 0, imm: 0, verts: 0 };
  try {
    mgl = new G.MiniGL(glCanvas);
    tex = new G.TexManager(mgl, pak);
    const TEXTURES = [
      ['data/textures/sphere.jpg'], ['data/textures/gothickiemura02.jpg'],
      ['data/textures/kalatus1-01.png'], ['data/textures/flare02.jpg'],
      ['data/textures/max_t3.jpg'], ['data/textures/max_t1.jpg'],
      ['data/textures/blend.tga'], ['data/textures/blend2.png'],
      ['data/textures/polka.png'], ['data/textures/polka.png', true],
      ['data/textures/y2.jpg'], ['data/textures/y6.jpg'], ['data/textures/y7.jpg'],
      ['data/textures/t1a.jpg'], ['data/textures/t1b.jpg'], ['data/textures/face.jpg'],
      ['data/textures/cred-saffron01.png'], ['data/textures/cred-yoghurt01.png'],
      ['data/textures/cred-radixlluvia01.png'],
    ];
    for (const [n, m] of TEXTURES) await tex.preload(n, m);

    // instrument the engine for the HUD
    const origDE = mgl.drawElements.bind(mgl);
    mgl.drawElements = (p, u, i) => { counters.draws++; counters.verts += i.length; origDE(p, u, i); };
    const origEnd = mgl.end.bind(mgl);
    mgl.end = () => { counters.imm++; counters.verts += mgl.immCount; origEnd(); };
  } catch (err) {
    glFailed = err.message;
  }

  const VIEWERS = [
    { key: 'spinzoom', s: 0, e: 24.5, make: () => new G.SpinZoom(mgl, tex) },
    { key: 'shadeball', s: 24.5, e: 48.5, make: () => new G.ShadeBall(mgl, tex) },
    { key: 'splines', s: 48.5, e: 67.7, make: () => new G.Splines(mgl, tex) },
    {
      key: 'ffdenv', s: 67.7, e: 87, make: () => new G.FFDEnv(mgl, tex, scenes.meta),
      overlay: { label: 'layer BANDS on top', s: 69, e: 85, stateful: true, make: () => new G.Bands(mgl, tex), draw: (inst, abs) => inst.do(abs) },
    },
    { key: 'energystream', s: 87, e: 145, make: () => new G.EnergyStream(mgl, tex) },
    {
      key: 'tubes', s: 145, e: 165, make: () => new G.Tubes(mgl, tex),
      extras: [
        { label: 'seed u', min: 3, max: 10, value: 4, key: 'su' },
        { label: 'seed v', min: 3, max: 10, value: 6, key: 'sv' },
      ],
      applyExtras: (inst, p) => inst.sobject.setSeed(p.su, p.sv),
    },
    {
      key: 'polkalike', s: 165, e: 203, stateful: true, make: () => new G.PolkaLike(mgl, tex),
      overlay: { label: 'layer TREE on top', s: 186, e: 201, make: () => new G.Tree(mgl, tex, scenes.kbuu), draw: (inst, abs) => inst.do(abs, 186) },
    },
    { key: 'facemorph', s: 203, e: 222.5, make: () => new G.FaceMorph(mgl, tex, scenes.faces) },
    { key: 'thing', s: 222.5, e: 254, make: () => new G.Thing(mgl, tex) },
  ];

  let active = null;
  const onTimeHooks = {}; // key -> [fn(localTime)] for plot/lab playheads
  const addTimeHook = (key, fn) => { (onTimeHooks[key] = onTimeHooks[key] || []).push(fn); };
  // secondary scripts (math labs) register their own playhead listeners here
  window.__teslaLabs = { onTime: addTimeHook, scenes, reducedMotion };

  function buildViewer(def) {
    const host = document.querySelector('[data-viewer="' + def.key + '"]');
    if (!host) return;
    def.host = host;
    def.t = 0.01;
    def.playing = false;
    def.wire = false;
    def.overlayOn = !!def.overlay;
    def.params = {};

    const hole = document.createElement('div');
    hole.className = 'screenhole';
    const ph = document.createElement('div');
    ph.className = 'placeholder';
    ph.textContent = glFailed ? ('webgl2 unavailable: ' + glFailed) : 'scroll here to activate';
    hole.appendChild(ph);
    host.appendChild(hole);
    def.hole = hole;

    const bar = document.createElement('div');
    bar.className = 'controls';

    const play = document.createElement('button');
    play.textContent = 'play';
    play.addEventListener('click', () => {
      def.playing = !def.playing;
      play.textContent = def.playing ? 'pause' : 'play';
    });
    bar.appendChild(play);
    def.playBtn = play;

    const range = document.createElement('input');
    range.type = 'range';
    range.min = '0';
    range.max = String(def.e - def.s);
    range.step = '0.01';
    range.value = String(def.t);
    range.setAttribute('aria-label', def.key + ' time');
    range.addEventListener('input', () => { setTime(def, parseFloat(range.value)); });
    bar.appendChild(range);
    def.range = range;

    const tlabel = document.createElement('span');
    tlabel.className = 'tlabel';
    bar.appendChild(tlabel);
    def.tlabel = tlabel;

    const wire = document.createElement('label');
    const wcb = document.createElement('input');
    wcb.type = 'checkbox';
    wcb.addEventListener('change', () => { def.wire = wcb.checked; });
    wire.appendChild(wcb);
    wire.appendChild(document.createTextNode('wireframe'));
    bar.appendChild(wire);

    if (def.overlay) {
      const ol = document.createElement('label');
      const ocb = document.createElement('input');
      ocb.type = 'checkbox';
      ocb.checked = true;
      ocb.addEventListener('change', () => { def.overlayOn = ocb.checked; });
      ol.appendChild(ocb);
      ol.appendChild(document.createTextNode(def.overlay.label));
      bar.appendChild(ol);
    }

    if (def.extras) {
      for (const ex of def.extras) {
        def.params[ex.key] = ex.value;
        const l = document.createElement('label');
        l.appendChild(document.createTextNode(ex.label + ' '));
        const r = document.createElement('input');
        r.type = 'range';
        r.min = String(ex.min);
        r.max = String(ex.max);
        r.step = '1';
        r.value = String(ex.value);
        r.style.width = '90px';
        const val = document.createElement('span');
        val.textContent = String(ex.value);
        r.addEventListener('input', () => {
          def.params[ex.key] = parseInt(r.value, 10);
          val.textContent = r.value;
        });
        l.appendChild(r);
        l.appendChild(val);
        bar.appendChild(l);
      }
    }

    const hud = document.createElement('div');
    hud.className = 'hud';
    bar.appendChild(hud);
    def.hud = hud;

    host.appendChild(bar);

    new IntersectionObserver((entries) => {
      for (const en of entries) {
        if (en.isIntersecting && en.intersectionRatio > 0.35) activate(def);
        else if (active === def && !en.isIntersecting) def.playing = false, def.playBtn.textContent = 'play';
      }
    }, { threshold: [0, 0.35, 0.7] }).observe(hole);
  }

  function setTime(def, t) {
    if (t < def.t - 0.25) {
      // scrubbing backwards: rebuild stateful effects so integration restarts
      if (def.stateful) def.instance = def.make();
      if (def.overlay && def.overlay.stateful) def.overlayInstance = def.overlay.make();
    }
    def.t = t;
  }

  function activate(def) {
    if (glFailed || active === def) return;
    active = def;
    def.hole.appendChild(glCanvas);
  }

  function renderActive(dt) {
    const def = active;
    if (!def || !mgl) return;
    if (!def.instance) def.instance = def.make();
    if (def.overlay && !def.overlayInstance) def.overlayInstance = def.overlay.make();

    if (def.playing) {
      let t = def.t + dt;
      if (t >= def.e - def.s) {
        t = 0.01;
        if (def.stateful) def.instance = def.make();
        if (def.overlay && def.overlay.stateful) def.overlayInstance = def.overlay.make();
      }
      def.t = t;
      def.range.value = String(t);
    }

    if (def.applyExtras) def.applyExtras(def.instance, def.params);

    counters.draws = 0; counters.imm = 0; counters.verts = 0;
    mgl.wireframe = def.wire;
    mgl.clear();
    const abs = def.s + def.t;
    try {
      def.instance.do(abs, def.s);
      if (def.overlay && def.overlayOn && abs > def.overlay.s && abs < def.overlay.e) {
        def.overlay.draw(def.overlayInstance, abs);
      }
    } catch (err) {
      def.hud.textContent = 'effect error: ' + err.message;
      def.playing = false;
      return;
    }
    mgl.wireframe = false;

    def.tlabel.textContent = 't=' + def.t.toFixed(2) + 's · demo ' + fmtTime(abs);
    def.hud.innerHTML = 'debug — array draws <b>' + counters.draws +
      '</b> · immediate batches <b>' + counters.imm +
      '</b> · vertices/frame <b>' + counters.verts + '</b>';

    const hooks = onTimeHooks[def.key];
    if (hooks) for (const fn of hooks) fn(def.t);
  }

  for (const def of VIEWERS) buildViewer(def);

  let lastFrame = performance.now();
  (function glLoop(now) {
    const dt = Math.min((now - lastFrame) / 1000, 0.1);
    lastFrame = now;
    renderActive(dt);
    requestAnimationFrame(glLoop);
  })(lastFrame);

  // ---------- debug plots ----------
  const clamp01 = (v) => (v < 0 ? 0 : v > 1 ? 1 : v);

  function plotEnvelope(canvasSel, key, domain, curves, opts = {}) {
    const canvas = $(canvasSel);
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const W = canvas.width, H = canvas.height, pad = 26;
    const xOf = (t) => pad + ((t - domain[0]) / (domain[1] - domain[0])) * (W - pad * 2);
    const yOf = (v) => H - pad - v * (H - pad * 2);
    let playhead = null;

    function draw() {
      ctx.clearRect(0, 0, W, H);
      ctx.strokeStyle = '#1b2431';
      ctx.lineWidth = 1;
      ctx.beginPath();
      for (let v = 0; v <= 1; v += 0.5) { ctx.moveTo(pad, yOf(v)); ctx.lineTo(W - pad, yOf(v)); }
      const tickStep = opts.tickStep || 5;
      ctx.fillStyle = '#67737f';
      ctx.font = '10px ui-monospace, monospace';
      for (let t = Math.ceil(domain[0]); t <= domain[1]; t += tickStep) {
        ctx.moveTo(xOf(t), yOf(0)); ctx.lineTo(xOf(t), yOf(0) + 4);
        ctx.fillText(t + 's', xOf(t) - 8, H - 8);
      }
      ctx.stroke();
      for (const c of curves) {
        ctx.strokeStyle = c.color;
        ctx.lineWidth = 1.6;
        ctx.beginPath();
        const steps = 480;
        for (let i = 0; i <= steps; i++) {
          const t = domain[0] + (i / steps) * (domain[1] - domain[0]);
          const v = clamp01(c.fn(t));
          i === 0 ? ctx.moveTo(xOf(t), yOf(v)) : ctx.lineTo(xOf(t), yOf(v));
        }
        ctx.stroke();
        if (c.label) {
          ctx.fillStyle = c.color;
          ctx.fillText(c.label, pad + (c.labelX || 4), 16 + (c.labelY || 0));
        }
      }
      if (playhead !== null) {
        ctx.strokeStyle = '#f0b46a';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(xOf(playhead), yOf(0));
        ctx.lineTo(xOf(playhead), yOf(1));
        ctx.stroke();
      }
    }
    draw();
    if (key) {
      addTimeHook(key, (t) => { playhead = t; draw(); });
    }
  }

  // SpinZoom envelope (unrTab)
  const SZ_ENV = [[0, 0], [3.3, 0], [6.5, 1], [8, 1], [9, 0], [10, 0], [13, 1], [15, 0], [16, 1], [20, 0], [22, 1], [24, 0]];
  const piecewise = (pts) => (t) => {
    if (t <= pts[0][0]) return pts[0][1];
    for (let i = 0; i < pts.length - 1; i++) {
      if (t >= pts[i][0] && t < pts[i + 1][0]) {
        const f = (t - pts[i][0]) / (pts[i + 1][0] - pts[i][0]);
        return pts[i][1] + (pts[i + 1][1] - pts[i][1]) * f;
      }
    }
    return pts[pts.length - 1][1];
  };
  plotEnvelope('#plot-spinzoom', 'spinzoom', [0, 24.5],
    [{ color: '#6fc3ff', fn: piecewise(SZ_ENV), label: 'unrMult' }]);

  // ShadeBall keys
  const SB_KEYS = [[0, 0], [1.5, 1], [5, 1], [7, 0], [9, 1], [14.5, 1], [16, 0], [18, 1], [20, 1], [24, 0]];
  plotEnvelope('#plot-shadeball', 'shadeball', [0, 24],
    [{ color: '#8ad0c8', fn: piecewise(SB_KEYS), label: 'alpha' }]);

  // EnergyStream master alpha (the nested clamp cascade, reimplemented)
  function esAlpha(t) {
    const C = 15, C1 = 25, C2 = 38, C3 = 58;
    if (t <= C) return clamp01((C - t) * 0.5) * (Math.sin(t * 2) * 0.25 + 0.75);
    let a = clamp01(t - C);
    if (t > C1) {
      a = clamp01(a * (t - C1));
      if (t > C2) {
        a = clamp01(a * (t - C2));
        if (t > C3) a = clamp01(a * (1 + C3 - t));
      } else {
        a = clamp01(a * (C2 - t));
      }
    } else {
      a = clamp01(a * (C1 - t));
    }
    return a;
  }
  plotEnvelope('#plot-energystream', 'energystream', [0, 58],
    [{ color: '#f0b46a', fn: esAlpha, label: 'alpha' }], { tickStep: 10 });

  // Thing credit layers
  function thingGlobal(t) { return t > 26.5 ? clamp01(1 + 26.5 - t) : 1; }
  plotEnvelope('#plot-thing', 'thing', [0, 31.5], [
    { color: '#f0e28a', fn: (t) => (t < 8 ? clamp01((8 - t) * 0.3) : 0) * thingGlobal(t), label: 'saffron', labelY: 0 },
    { color: '#ff9bb0', fn: (t) => (t < 8 ? 1 - clamp01((8 - t) * 0.3) : 1 - clamp01((t - 16) * 0.3)) * thingGlobal(t), label: 'radix/lluvia', labelY: 14 },
    { color: '#b48cff', fn: (t) => (t >= 8 ? clamp01((t - 16) * 0.3) : 0) * thingGlobal(t), label: 'yoghurt', labelY: 28 },
  ]);

  // Splines trail path (x/y projection of trail 0)
  (function plotSplinesPath() {
    const canvas = $('#plot-splines');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const W = canvas.width, H = canvas.height;
    ctx.clearRect(0, 0, W, H);
    const px = (t) => Math.sin(t * 1.2) * 100 + Math.sin(t * 2.3 + 1) * 100;
    const py = (t) => Math.sin(t * 2.3) * 100 + Math.sin(t * 2.13 + 1) * 100;
    ctx.strokeStyle = 'rgba(240,226,138,0.75)';
    ctx.lineWidth = 1.2;
    ctx.beginPath();
    for (let i = 0; i <= 4000; i++) {
      const t = (i / 4000) * 20;
      const x = W / 2 + px(t) * (W / 460);
      const y = H / 2 - py(t) * (H / 460);
      i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
    }
    ctx.stroke();
  })();

  // PolkaLike emitter paths, three phases
  (function plotPolkaPaths() {
    const canvas = $('#plot-polkalike');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const W = canvas.width, H = canvas.height;
    ctx.clearRect(0, 0, W, H);
    const phases = [
      { color: 'rgba(255,155,176,0.85)', fx: (p) => 10 * Math.cos(p) + 10 * Math.sin(p * 1.32), fy: (p) => 15 * Math.sin(p) + 5 * Math.sin(p * 0.89), label: 'phase 1' },
      { color: 'rgba(111,195,255,0.85)', fx: (p) => 15 * Math.cos(p) + 15 * Math.sin(p * 2.32), fy: (p) => 30 * Math.sin(p) + 15 * Math.sin(p * 1.89), label: 'phase 2' },
      { color: 'rgba(192,240,122,0.85)', fx: (p) => 30 * Math.cos(p) + 15 * Math.sin(p * 0.72), fy: (p) => 25 * Math.sin(p) + 19 * Math.sin(p * 0.59), label: 'phase 3' },
    ];
    ctx.font = '11px ui-monospace, monospace';
    phases.forEach((ph, k) => {
      const ox = W / 6 + (k * W) / 3;
      ctx.strokeStyle = ph.color;
      ctx.lineWidth = 1;
      ctx.beginPath();
      for (let i = 0; i <= 2400; i++) {
        const p = (i / 2400) * 42;
        const x = ox + ph.fx(p) * 2.6;
        const y = H / 2 - 8 - ph.fy(p) * 2.6;
        i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
      }
      ctx.stroke();
      ctx.fillStyle = ph.color;
      ctx.fillText(ph.label, ox - 22, H - 10);
    });
  })();

  // Tree FFD lattice, animated 2D slice
  (function plotTreeLattice() {
    const canvas = $('#plot-tree');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const W = canvas.width, H = canvas.height;
    let visible = false, T = 3;
    new IntersectionObserver((es) => { visible = es[0].isIntersecting; }).observe(canvas);

    function draw() {
      ctx.clearRect(0, 0, W, H);
      const pts = [];
      for (let y = 0; y < 5; y++) {
        for (let x = 0; x < 5; x++) {
          const dx = (x * 4 - 8) * (Math.sin(T * 1.33 + y * 4) * 0.2 + 0.8);
          const dy = (y * 4 - 8) * (Math.cos(T * 2 + x * 10) * 0.2 + 0.8);
          pts.push([W / 2 + dx * 14, H / 2 - dy * 14]);
        }
      }
      ctx.strokeStyle = 'rgba(192,240,122,0.5)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      for (let y = 0; y < 5; y++) {
        for (let x = 0; x < 5; x++) {
          const p = pts[y * 5 + x];
          if (x < 4) { const q = pts[y * 5 + x + 1]; ctx.moveTo(p[0], p[1]); ctx.lineTo(q[0], q[1]); }
          if (y < 4) { const q = pts[(y + 1) * 5 + x]; ctx.moveTo(p[0], p[1]); ctx.lineTo(q[0], q[1]); }
        }
      }
      ctx.stroke();
      ctx.fillStyle = '#c0f07a';
      for (const p of pts) { ctx.beginPath(); ctx.arc(p[0], p[1], 2.4, 0, 7); ctx.fill(); }
    }

    (function loop() {
      if (visible) {
        if (!reducedMotion) T += 0.016;
        draw();
      }
      requestAnimationFrame(loop);
    })();
    draw();
  })();

  // ---------- done ----------
  setGate('ready');
  const gate = $('#gate');
  gate.classList.add('done');
  setTimeout(() => gate.remove(), 600);
})();
