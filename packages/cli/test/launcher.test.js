import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const binPath = path.resolve(__dirname, '../bin/xgauntlet.js');

test('xgauntlet.js launcher status output', () => {
  const output = execFileSync(process.execPath, [binPath, 'status'], {
    encoding: 'utf8',
  });
  assert.match(output, /xgauntlet/i);
});

test('xgauntlet.js launcher doctor output', () => {
  const output = execFileSync(process.execPath, [binPath, 'doctor'], {
    encoding: 'utf8',
  });
  assert.match(output, /doctor/i);
  assert.match(output, /Platform|OS/i);
});

test('xgauntlet.js launcher init help output', () => {
  const output = execFileSync(process.execPath, [binPath, 'init', '--help'], {
    encoding: 'utf8',
  });
  assert.match(output, /Initialize in-repo governance files/i);
  assert.match(output, /--force/i);
  assert.match(output, /--dry-run/i);
});

test('xgauntlet.js launcher check-release output', () => {
  const output = execFileSync(process.execPath, [binPath, 'check-release', '--workspace', path.resolve(__dirname, '../../..')], {
    encoding: 'utf8',
  });
  assert.match(output, /Release Readiness Gatekeeper/i);
  assert.match(output, /RELEASE READY/i);
});
