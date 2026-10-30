const fs = require('node:fs');
const path = require('node:path');
const root = path.join(__dirname, '..');
const cargo = fs.readFileSync(path.join(root, 'Cargo.toml'),'utf8');
const version = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
if (!version) throw Error('No Cargo version');
for (const dir of ['cli','platform/darwin-arm64','platform/darwin-x64']) {
  const manifest = require(path.join(root,'npm',dir,'package.json'));
  if (manifest.version !== version) throw Error(`${dir}: npm version ${manifest.version} != Cargo ${version}`);
}
for (const dir of ['darwin-arm64','darwin-x64']) {
  const bin = path.join(root,'npm','platform',dir,'bin','smartcopy');
  if (!fs.statSync(bin).isFile()) throw Error(`Missing native binary: ${bin}`);
}
if (process.env.GITHUB_REF_TYPE === 'tag' && process.env.GITHUB_REF_NAME !== `v${version}`) {
  throw Error(`Tag must equal v${version}`);
}
console.log('Native binaries and npm manifests match Cargo version', version);
