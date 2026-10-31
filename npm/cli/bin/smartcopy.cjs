#!/usr/bin/env node
'use strict';
const { spawnSync } = require('node:child_process');
const { join } = require('node:path');
const { createRequire } = require('node:module');
const req = createRequire(__filename);
if (process.platform !== 'darwin') {
  console.error('Smart Copy currently supports macOS only. Windows support is planned.');
  process.exit(1);
}
const name = `@arifzulfugarov/smartcopy-darwin-${process.arch}`;
let bin;
try { bin = join(req.resolve(`${name}/package.json`), '..', 'bin', 'smartcopy'); }
catch (_) {
  console.error(`Missing ${name}. Reinstall with optional dependencies enabled (npm install -g @arifzulfugarov/smartcopy).`);
  process.exit(1);
}
const result = spawnSync(bin, process.argv.slice(2), { stdio: 'inherit' });
if (result.error) { console.error(`Could not launch Smart Copy: ${result.error.message}`); process.exit(1); }
process.exit(result.status ?? 1);
