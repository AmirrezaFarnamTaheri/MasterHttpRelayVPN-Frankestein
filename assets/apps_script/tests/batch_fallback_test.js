'use strict';

const fs = require('fs');
const path = require('path');

const relays = [
  'Code.gs',
  'CodeFull.gs',
  'CodeCloudflareWorker.gs',
];

function check(label, condition, detail = '') {
  if (!condition) {
    throw new Error(`${label}${detail ? `: ${detail}` : ''}`);
  }
}

for (const relay of relays) {
  const src = fs.readFileSync(path.join(__dirname, '..', relay), 'utf8');
  check(`${relay} declares safe replay methods`, src.includes('const SAFE_REPLAY_METHODS'));
  check(`${relay} validates bad batch items`, src.includes('"bad item"'));
  check(`${relay} catches fetchAll failure`, src.includes('catch (err)'));
  check(`${relay} skips unsafe replay`, src.includes('unsafe method not replayed'));
  check(`${relay} uses original-index response map`, src.includes('var responseMap = {}'));
  check(`${relay} maps fetchAll responses by original index`, src.includes('responseMap[fetchArgs[a]._i] = responses[a]'));
  check(`${relay} maps fallback responses by original index`, src.includes('] = UrlFetchApp.fetch(fallbackUrl, fallbackOpts)'));
}

console.log('batch fallback source checks ok');
