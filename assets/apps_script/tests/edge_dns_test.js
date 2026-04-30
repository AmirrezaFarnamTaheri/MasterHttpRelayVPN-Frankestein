'use strict';

const fs = require('fs');
const path = require('path');

const src = fs.readFileSync(path.join(__dirname, '..', 'CodeFull.gs'), 'utf8');
const names = [
  '_dnsSkipName',
  '_dnsParseQuestion',
  '_dnsMinTtl',
  '_dnsRewriteTxid',
  '_spliceTunnelResults',
];

let bundle = '';
for (const name of names) {
  const match = src.match(new RegExp(`function ${name}\\b[\\s\\S]*?\\n\\}\\n`, 'g'));
  if (!match) throw new Error(`helper not found in CodeFull.gs: ${name}`);
  bundle += match[0] + '\n';
}
bundle += `return { ${names.join(', ')} };`;
const ctx = new Function(bundle)();

function check(label, condition, detail = '') {
  if (!condition) {
    throw new Error(`${label}${detail ? `: ${detail}` : ''}`);
  }
}

const queryA = Buffer.from([
  0x12, 0x34, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x07, 0x65, 0x78, 0x61,
  0x6d, 0x70, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d,
  0x00, 0x00, 0x01, 0x00, 0x01,
]);
const parsed = ctx._dnsParseQuestion(queryA);
check('parse txid', parsed.txid === 0x1234);
check('parse qname', parsed.qname === 'example.com', parsed.qname);
check('parse qtype', parsed.qtype === 1);

const queryAAAAUpper = Buffer.from([
  0xab, 0xcd, 0x01, 0x00, 0x00, 0x01, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x07, 0x45, 0x58, 0x41,
  0x4d, 0x50, 0x4c, 0x45, 0x03, 0x43, 0x4f, 0x4d,
  0x00, 0x00, 0x1c, 0x00, 0x01,
]);
const parsedUpper = ctx._dnsParseQuestion(queryAAAAUpper);
check('case-fold qname', parsedUpper.qname === 'example.com', parsedUpper.qname);
check('AAAA qtype', parsedUpper.qtype === 28);

const rewritten = ctx._dnsRewriteTxid(queryA, 0xdead);
check('rewrite hi', (rewritten[0] & 0xff) === 0xde);
check('rewrite lo', (rewritten[1] & 0xff) === 0xad);
check('source not mutated', queryA[0] === 0x12 && queryA[1] === 0x34);
for (let i = 2; i < queryA.length; i++) {
  check(`rewrite keeps byte ${i}`, (rewritten[i] & 0xff) === queryA[i]);
}

const reply = Buffer.from([
  0x12, 0x34, 0x81, 0x80, 0x00, 0x01, 0x00, 0x01,
  0x00, 0x00, 0x00, 0x00, 0x07, 0x65, 0x78, 0x61,
  0x6d, 0x70, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d,
  0x00, 0x00, 0x01, 0x00, 0x01, 0xc0, 0x0c,
  0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x01, 0x2c,
  0x00, 0x04, 0x5d, 0xb8, 0xd8, 0x22,
]);
check('min TTL with pointer', ctx._dnsMinTtl(reply) === 300);

const highBitTtlReply = Buffer.from(reply);
highBitTtlReply[35] = 0x80;
highBitTtlReply[36] = 0x00;
highBitTtlReply[37] = 0x00;
highBitTtlReply[38] = 0x00;
check('high-bit TTL clamps to 0', ctx._dnsMinTtl(highBitTtlReply) === 0);

check('truncated query rejected', ctx._dnsParseQuestion(Buffer.from([0, 1, 2])) === null);
check(
  'question compression rejected',
  ctx._dnsParseQuestion(Buffer.from([
    0, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0xc0, 0x0c, 0, 1, 0, 1,
  ])) === null,
);

const all = new Array(5);
all[1] = { sid: 'cache' };
all[3] = { sid: 'doh' };
const merged = ctx._spliceTunnelResults(
  [0, 2, 4],
  [{ sid: 'tcp0' }, { sid: 'tcp2' }, { sid: 'tcp4' }],
  all,
);
check('splice slot 0', merged[0].sid === 'tcp0');
check('splice slot 1', merged[1].sid === 'cache');
check('splice slot 4', merged[4].sid === 'tcp4');
check('splice returns same array', merged === all);

console.log('edge_dns_test.js: all tests passed');
