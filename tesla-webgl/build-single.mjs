#!/usr/bin/env node
// Builds a fully self-contained single-file build of the Tesla port:
// all JS modules bundled into one scope-per-module namespace, and
// data.pak plus the soundtrack embedded as base64. Useful for hosting
// environments that only accept one HTML file (e.g. Claude Artifacts).
//
// Usage: node build-single.mjs [out.html]

import { readFileSync, writeFileSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const root = dirname(fileURLToPath(import.meta.url));
const out = process.argv[2] || join(root, 'tesla-single.html');

// dependency order
const MODULES = [
  'js/mathlib.js',
  'js/minigl.js',
  'js/pak.js',
  'js/t3ds.js',
  'js/effects/common.js',
  'js/effects/tessellator.js',
  'js/effects/spinzoom.js',
  'js/effects/shadeball.js',
  'js/effects/splines.js',
  'js/effects/ffdenv.js',
  'js/effects/bands.js',
  'js/effects/energystream.js',
  'js/effects/tubes.js',
  'js/effects/polkalike.js',
  'js/effects/tree.js',
  'js/effects/facemorph.js',
  'js/effects/thing.js',
  'js/demo.js',
];

// Convert one ES module into an IIFE over the shared namespace `G`:
// imports become destructuring from G, exports get assigned back onto G.
function transform(src) {
  const exported = [];

  // import { A, B } from '...';  ->  const { A, B } = G;
  src = src.replace(/import\s*\{([^}]*)\}\s*from\s*['"][^'"]+['"];?/g, (_, names) => {
    return `const {${names}} = G;`;
  });

  src = src.replace(/^export\s+(class|function|const|let)\s+([A-Za-z0-9_$]+)/gm, (_, kind, name) => {
    exported.push(name);
    return `${kind} ${name}`;
  });

  return `(() => {\n${src}\nObject.assign(G, { ${exported.join(', ')} });\n})();`;
}

const bundled = MODULES
  .map((m) => `// ===== ${m} =====\n` + transform(readFileSync(join(root, m), 'utf8')))
  .join('\n\n');

const pakB64 = readFileSync(join(root, 'data/data.pak')).toString('base64');
const mp3B64 = readFileSync(join(root, 'data/tournesol.mp3')).toString('base64');

const html = `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Tesla by Sunflower — WebGL port</title>
<meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover">
<style>
  html, body {
    margin: 0;
    height: 100%;
    background: #000;
    color: #9aa;
    font-family: "Lucida Console", Monaco, monospace;
    overflow: hidden;
    -webkit-text-size-adjust: 100%;
  }
  #wrap {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 16px;
  }
  #screen {
    background: #000;
    width: min(100vw, calc(100vh * 4 / 3));
    aspect-ratio: 4 / 3;
    display: none;
    touch-action: none;
  }
  #panel { text-align: center; max-width: 34em; line-height: 1.6; padding: 0 16px; }
  #panel h1 {
    font-size: 18px;
    font-weight: normal;
    letter-spacing: 0.5em;
    color: #dde;
    margin-bottom: 4px;
  }
  #panel .sub { font-size: 12px; }
  #status { font-size: 12px; color: #667; min-height: 1.6em; }
  #play {
    margin-top: 12px;
    padding: 12px 46px;
    background: transparent;
    color: #dde;
    border: 1px solid #556;
    font: inherit;
    letter-spacing: 0.3em;
    cursor: pointer;
    display: none;
  }
  #play:hover { background: #112; }
  #hint { font-size: 11px; color: #445; margin-top: 10px; }
  #problem {
    position: fixed;
    left: 0; right: 0; bottom: 0;
    padding: 8px;
    text-align: center;
    font-size: 12px;
    color: #faa;
    background: rgba(40, 0, 0, 0.8);
    display: none;
  }
</style>
</head>
<body>
<div id="wrap">
  <div id="panel">
    <h1>T E S L A</h1>
    <div class="sub">sunflower, 2000 &mdash; javascript / webgl port</div>
    <div id="status">loading...</div>
    <button id="play">play</button>
    <div id="hint">runs 4:14 &middot; sound on &middot; tap to stop</div>
  </div>
  <canvas id="screen" width="960" height="720"></canvas>
</div>
<div id="problem"></div>
<script>
const G = {};
${bundled}

const PAK_B64 = "${pakB64}";
const MP3_B64 = "${mp3B64}";

function b64ToBuffer(b64) {
  const bin = atob(b64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return bytes.buffer;
}

(async () => {
  const canvas = document.getElementById('screen');
  const panel = document.getElementById('panel');
  const statusEl = document.getElementById('status');
  const playBtn = document.getElementById('play');

  const problemEl = document.getElementById('problem');
  const showProblem = (msg) => {
    problemEl.textContent = msg;
    problemEl.style.display = 'block';
  };
  window.addEventListener('error', (e) => showProblem('error: ' + e.message));

  const demo = new G.Demo(canvas, (msg) => { statusEl.textContent = msg; }, {
    pakBuffer: b64ToBuffer(PAK_B64),
    audioSrc: 'data:audio/mpeg;base64,' + MP3_B64,
    onProblem: showProblem,
  });
  window.demo = demo;

  try {
    await demo.load();
    statusEl.textContent = 'ready — 254 seconds, sound on';
    playBtn.style.display = 'inline-block';
  } catch (err) {
    statusEl.textContent = 'error: ' + err.message;
    throw err;
  }

  const stop = () => {
    demo.stop();
    canvas.style.display = 'none';
    panel.style.display = 'block';
    statusEl.textContent = 'stopped — tap play to restart';
  };

  playBtn.addEventListener('click', () => {
    panel.style.display = 'none';
    canvas.style.display = 'block';
    demo.start();
  });

  canvas.addEventListener('pointerdown', stop);
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
console.log(`wrote ${out} (${(html.length / 1024 / 1024).toFixed(1)} MB)`);
