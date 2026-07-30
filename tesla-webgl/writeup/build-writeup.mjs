#!/usr/bin/env node
// Builds the interactive "making of" write-up as one self-contained HTML:
// prose + CSS + the bundled port engine + data.pak embedded as base64
// (no soundtrack — the write-up's viewers are silent).
//
// Usage: node writeup/build-writeup.mjs [out.html]

import { readFileSync, writeFileSync } from 'fs';
import { dirname, join } from 'path';
import { fileURLToPath } from 'url';

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, '..');
const out = process.argv[2] || join(here, 'tesla-writeup.html');

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
];

function transform(src) {
  const exported = [];
  src = src.replace(/import\s*\{([^}]*)\}\s*from\s*['"][^'"]+['"];?/g, (_, names) => `const {${names}} = G;`);
  src = src.replace(/^export\s+(class|function|const|let)\s+([A-Za-z0-9_$]+)/gm, (_, kind, name) => {
    exported.push(name);
    return `${kind} ${name}`;
  });
  return `(() => {\n${src}\nObject.assign(G, { ${exported.join(', ')} });\n})();`;
}

const engine = MODULES
  .map((m) => `// ===== ${m} =====\n` + transform(readFileSync(join(root, m), 'utf8')))
  .join('\n\n');

const css = readFileSync(join(here, 'style.css'), 'utf8');
const body = readFileSync(join(here, 'article-body.html'), 'utf8');
const article = readFileSync(join(here, 'article.js'), 'utf8');
const pakB64 = readFileSync(join(root, 'data/data.pak')).toString('base64');

for (const [name, text] of [['engine', engine], ['article.js', article]]) {
  if (text.includes('</scr' + 'ipt>')) throw new Error(name + ' contains a script-closing tag');
}

const html = [
  '<!DOCTYPE html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n',
  '<title>Porting Tesla by Sunflower to WebGL</title>\n',
  '<meta name="viewport" content="width=device-width, initial-scale=1">\n',
  '<style>\n', css, '\n</style>\n</head>\n<body>\n',
  body,
  '\n<script>\nconst G = {};\n', engine, '\nwindow.G = G;\n</script>\n',
  '<script>\nconst PAK_B64 = "', pakB64, '";\n</script>\n',
  '<script>\n', article, '\n</script>\n',
  '</body>\n</html>\n',
].join('');

writeFileSync(out, html);
console.log(`wrote ${out} (${(html.length / 1024 / 1024).toFixed(1)} MB)`);
