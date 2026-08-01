#!/usr/bin/env node
// Self-contained single-file build of the Nowhere port: every module
// bundled into one namespace and the whole data/ tree embedded as
// base64. Shaped for a phone — the canvas runs at the demo's native
// 640x480, the readout is off until you tap for it, and nothing scrolls.
//
// Usage: node build-single.mjs [out.html]

import { readFileSync, writeFileSync, readdirSync, statSync } from 'fs';
import { dirname, join, relative } from 'path';
import { fileURLToPath } from 'url';

const root = dirname(fileURLToPath(import.meta.url));
const out = process.argv[2] || join(root, 'nowhere-single.html');

const MODULES = [
  'js/mathlib.js',
  'js/minigl.js',
  'js/zeus.js',
  'js/effects/zeusplay.js',
  'js/effects/backgr.js',
  'js/effects/anim.js',
  'js/effects/sphere.js',
  'js/effects/credits.js',
  'js/effects/flower.js',
  'js/demo.js',
];

// Turn an ES module into an IIFE that publishes its exports on G.
function transform(src) {
  const exported = [];
  src = src.replace(/import\s*\{([^}]*)\}\s*from\s*['"][^'"]+['"];?/g,
                    (_, names) => `const {${names}} = G;`);
  src = src.replace(/^export\s+(?:async\s+)?(class|function|const|let)\s+([A-Za-z0-9_$]+)/gm,
                    (m, kind, name) => {
                      exported.push(name);
                      return m.replace(/^export\s+/, '');
                    });
  // `export { a, b };` — the names are already declared above
  src = src.replace(/^export\s*\{([^}]*)\};?/gm, (_, names) => {
    for (const n of names.split(',')) {
      const t = n.trim();
      if (t) exported.push(t);
    }
    return '';
  });
  return `(() => {\n${src}\nObject.assign(G, { ${[...new Set(exported)].join(', ')} });\n})();`;
}

const engine = MODULES
  .map((m) => `// ===== ${m} =====\n` + transform(readFileSync(join(root, m), 'utf8')))
  .join('\n\n');
if (engine.includes('</scr' + 'ipt>')) throw new Error('engine contains a script-closing tag');

const MIME = {
  '.json': 'application/json', '.jpg': 'image/jpeg', '.png': 'image/png',
  '.mp3': 'audio/mpeg', '.txt': 'text/plain',
};

function walk(dir, acc = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, acc);
    else acc.push(p);
  }
  return acc;
}

const embed = {};
let bytes = 0;
for (const p of walk(join(root, 'data'))) {
  const rel = relative(root, p).replace(/\\/g, '/');
  const ext = rel.slice(rel.lastIndexOf('.'));
  const mime = MIME[ext];
  if (!mime) continue;
  const buf = readFileSync(p);
  bytes += buf.length;
  embed[rel.replace(/^data\//, '')] = { mime, b64: buf.toString('base64') };
}

const html = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Nowhere by Threestate — WebGL port</title>
<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, viewport-fit=cover">
<meta name="apple-mobile-web-app-capable" content="yes">
<meta name="theme-color" content="#000000">
<style>
  * { -webkit-tap-highlight-color: transparent; }
  html, body {
    margin: 0; padding: 0; height: 100%; width: 100%; background: #000;
    color: #b9a06a; font-family: "Lucida Console", Menlo, monospace;
    overflow: hidden; overscroll-behavior: none;
    -webkit-text-size-adjust: 100%; -webkit-user-select: none; user-select: none;
  }
  #stage {
    position: fixed; inset: 0; display: none;
    align-items: center; justify-content: center; background: #000;
  }
  #screen { width: 100%; height: auto; max-height: 100%; aspect-ratio: 4 / 3; touch-action: none; display: block; }
  @media (orientation: portrait) { #screen { width: 100%; } }
  @media (orientation: landscape) { #screen { width: auto; height: 100%; } }
  #parts {
    position: fixed; left: max(10px, env(safe-area-inset-left)); bottom: max(10px, env(safe-area-inset-bottom));
    font-size: 10px; line-height: 1.7; color: #6a5c3a; text-shadow: 0 1px 2px #000;
    pointer-events: none; max-height: 60%; overflow: hidden; display: none;
  }
  #parts .on { color: #e8b850; }
  #panel {
    position: fixed; inset: 0; display: flex; flex-direction: column;
    align-items: center; justify-content: center; text-align: center;
    padding: 0 24px; box-sizing: border-box; gap: 6px;
  }
  h1 { font-size: clamp(15px, 5.2vw, 26px); font-weight: normal; letter-spacing: 0.42em;
       color: #f0c96a; margin: 0 0 6px 0; text-indent: 0.42em; white-space: nowrap; }
  .sub { font-size: clamp(11px, 3.2vw, 13px); color: #7a6740; line-height: 1.6; max-width: 30em; }
  #status { font-size: 12px; color: #6a5c3a; min-height: 1.6em; margin-top: 14px; }
  #play {
    margin-top: 10px; padding: 16px 54px; background: transparent; color: #f0c96a;
    border: 1px solid #5c4d2c; font: inherit; font-size: 15px; letter-spacing: 0.35em;
    text-indent: 0.35em; border-radius: 2px; display: none; cursor: pointer;
  }
  #play:active { background: #221a0a; }
  #hint { font-size: 11px; color: #4a3f26; margin-top: 14px; line-height: 1.7; }
  #tap {
    position: fixed; right: max(10px, env(safe-area-inset-right)); top: max(10px, env(safe-area-inset-top));
    width: 34px; height: 34px; border: 1px solid #3a3120; border-radius: 50%;
    color: #6a5c3a; font-size: 13px; line-height: 32px; text-align: center; display: none;
  }
</style>
</head>
<body>
<div id="panel">
  <h1>N O W H E R E</h1>
  <div class="sub">threestate &middot; takeover 2000</div>
  <div class="sub" style="margin-top:10px">
    A WebGL port. The demo's own script drives this, cue for cue, against
    the shipped soundtrack.
  </div>
  <div id="status">loading&hellip;</div>
  <button id="play">play</button>
  <div id="hint">3:26 &middot; turn the sound on &middot; landscape looks best<br>tap the screen to stop &middot; tap <b>i</b> for the part list</div>
</div>
<div id="stage"><canvas id="screen" width="640" height="480"></canvas></div>
<div id="parts"></div>
<div id="tap">i</div>
<script>
const G = {};
${engine}

const EMBED = ${JSON.stringify(embed)};

// Serve the embedded tree to the runtime: fetch() and Image both go
// through data: URIs, so nothing touches the network.
(() => {
  const realFetch = window.fetch.bind(window);
  window.fetch = (url, opts) => {
    const key = String(url).replace(/^data\\//, '');
    const e = EMBED[key];
    if (!e) return realFetch(url, opts);
    const bin = atob(e.b64);
    const buf = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) buf[i] = bin.charCodeAt(i);
    return Promise.resolve(new Response(buf, { headers: { 'Content-Type': e.mime } }));
  };
  const desc = Object.getOwnPropertyDescriptor(HTMLImageElement.prototype, 'src');
  Object.defineProperty(HTMLImageElement.prototype, 'src', {
    set(v) {
      const e = EMBED[String(v).replace(/^data\\//, '')];
      desc.set.call(this, e ? 'data:' + e.mime + ';base64,' + e.b64 : v);
    },
    get() { return desc.get.call(this); },
  });
  const RealAudio = window.Audio;
  window.Audio = function (src) {
    const e = EMBED[String(src || '').replace(/^data\\//, '')];
    return new RealAudio(e ? 'data:' + e.mime + ';base64,' + e.b64 : src);
  };
})();

(async () => {
  const canvas = document.getElementById('screen');
  const stage = document.getElementById('stage');
  const panel = document.getElementById('panel');
  const statusEl = document.getElementById('status');
  const playBtn = document.getElementById('play');
  const partsEl = document.getElementById('parts');
  const tapBtn = document.getElementById('tap');

  let showParts = false;
  const demo = new G.Demo(canvas, (m) => { statusEl.textContent = m; }, {
    onActiveChange(time, active) {
      if (!showParts) return;
      let html = 't = ' + time.toFixed(1) + 's<br>';
      for (const a of active) {
        html += '<span class="' + (a.ready ? 'on' : '') + '">' +
          (a.ready ? '\\u25b6 ' : '\\u00b7 ') + a.name + ' [' + a.effect + ']' +
          (a.ready ? (a.raw ? ' (scene only)' : '') : ' (not ported yet)') + '</span><br>';
      }
      partsEl.innerHTML = html;
    },
  });
  window.demo = demo;

  try {
    await demo.load();
    statusEl.textContent = 'ready';
    playBtn.style.display = 'inline-block';
  } catch (err) {
    statusEl.textContent = 'error: ' + err.message;
    throw err;
  }

  const stop = () => {
    demo.stop();
    stage.style.display = 'none';
    tapBtn.style.display = 'none';
    partsEl.style.display = 'none';
    panel.style.display = 'flex';
    statusEl.textContent = 'stopped \\u2014 tap play to restart';
  };

  playBtn.addEventListener('click', () => {
    panel.style.display = 'none';
    stage.style.display = 'flex';
    tapBtn.style.display = 'block';
    demo.start();
  });
  canvas.addEventListener('pointerdown', stop);
  tapBtn.addEventListener('pointerdown', (e) => {
    e.stopPropagation();
    showParts = !showParts;
    partsEl.style.display = showParts ? 'block' : 'none';
  });
  document.addEventListener('gesturestart', (e) => e.preventDefault());
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') stop();
    else if (e.key === 'ArrowRight') demo.seek(demo.getTime() + 5);
    else if (e.key === 'ArrowLeft') demo.seek(demo.getTime() - 5);
  });
})();
</script>
</body>
</html>
`;

writeFileSync(out, html);
console.log(`wrote ${out} (${(html.length / 1048576).toFixed(1)} MB from ${(bytes / 1048576).toFixed(1)} MB of assets)`);
