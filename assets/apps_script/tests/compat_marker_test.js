const fs = require('fs');
const path = require('path');

const root = path.resolve(__dirname, '..', '..', '..');

const helpers = [
  {
    file: 'assets/apps_script/Code.gs',
    kind: 'apps_script',
    features: ['single', 'batch', 'safe_fetchall_fallback', 'header_privacy'],
  },
  {
    file: 'assets/apps_script/CodeFull.gs',
    kind: 'apps_script_full',
    features: ['single', 'batch', 'full_tunnel', 'tunnel_batch', 'edge_dns_cache'],
  },
  {
    file: 'assets/apps_script/CodeCloudflareWorker.gs',
    kind: 'apps_script_cloudflare_worker',
    features: ['single', 'batch', 'cloudflare_worker_exit', 'safe_fetchall_fallback'],
  },
];

function check(label, ok) {
  if (!ok) {
    throw new Error(label);
  }
}

for (const helper of helpers) {
  const src = fs.readFileSync(path.join(root, helper.file), 'utf8');
  check(`${helper.file} has helper kind`, src.includes(`const HELPER_KIND = "${helper.kind}"`));
  check(`${helper.file} has helper version`, /const HELPER_VERSION = "\d{4}-\d{2}-\d{2}\.batch\d+"/.test(src));
  check(`${helper.file} has protocol marker`, src.includes('const HELPER_PROTOCOL = "mhrv-f.apps-script.v1"'));
  check(`${helper.file} has compat function`, src.includes('function _compatInfo()'));
  check(`${helper.file} exposes compat probe`, src.includes('e.parameter.compat === "1"'));
  check(`${helper.file} keeps decoy default`, src.includes('createTextOutput(DECOY_HTML)'));
  for (const feature of helper.features) {
    check(`${helper.file} declares feature ${feature}`, src.includes(`"${feature}"`));
  }
}

console.log('compat marker source checks ok');
