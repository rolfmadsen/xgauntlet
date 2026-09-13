import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import path from 'node:path';
import fs from 'node:fs';
import { fileURLToPath } from 'node:url';
import { getPlatformInfo } from '../bin/xgauntlet.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const binPath = path.resolve(__dirname, '../bin/xgauntlet.js');

test('xgauntlet.js getPlatformInfo resolves arm64 linux correctly', () => {
  const info = getPlatformInfo('linux', 'arm64');
  assert.equal(info.platform, 'linux');
  assert.equal(info.arch, 'arm64');
  assert.equal(info.isWindows, false);
  assert.equal(info.binName, 'xgauntlet');
  assert.equal(info.archiveName, 'xgauntlet-linux-arm64.tar.gz');
});

test('xgauntlet.js getPlatformInfo resolves all supported distribution platforms', () => {
  assert.equal(getPlatformInfo('linux', 'x64').archiveName, 'xgauntlet-linux-x64.tar.gz');
  assert.equal(getPlatformInfo('linux', 'arm64').archiveName, 'xgauntlet-linux-arm64.tar.gz');
  assert.equal(getPlatformInfo('darwin', 'arm64').archiveName, 'xgauntlet-darwin-arm64.tar.gz');
  assert.equal(getPlatformInfo('darwin', 'x64').archiveName, 'xgauntlet-darwin-x64.tar.gz');
  assert.equal(getPlatformInfo('win32', 'x64').archiveName, 'xgauntlet-win32-x64.zip');
});

test('release.yml contains linux arm64 build target and tarball', () => {
  const releaseYamlPath = path.resolve(__dirname, '../../../.github/workflows/release.yml');
  const releaseYaml = fs.readFileSync(releaseYamlPath, 'utf8');
  assert.match(releaseYaml, /aarch64-unknown-linux-gnu/);
  assert.match(releaseYaml, /xgauntlet-linux-arm64\.tar\.gz/);
});

test('ci.yml contains ubuntu-24.04-arm runner', () => {
  const ciYamlPath = path.resolve(__dirname, '../../../.github/workflows/ci.yml');
  const ciYaml = fs.readFileSync(ciYamlPath, 'utf8');
  assert.match(ciYaml, /ubuntu-24\.04-arm/);
});

test('xgauntlet.js launcher status output', () => {
  const output = execFileSync(process.execPath, [binPath, 'status'], {
    encoding: 'utf8',
  });
  assert.match(output, /xgauntlet/i);
});

test('xgauntlet.js launcher doctor output', () => {
  const output = execFileSync(process.execPath, [binPath, 'doctor', '--workspace', path.resolve(__dirname, '../../..')], {
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
