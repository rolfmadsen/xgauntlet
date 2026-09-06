#!/usr/bin/env node
/**
 * xgauntlet - Universal cross-platform verification gauntlet for AI coding agents.
 * Bootstrapper that resolves, caches, or downloads the native Rust engine binary.
 */

import { spawnSync, spawn, execFileSync } from 'node:child_process';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const pkgJsonPath = path.resolve(__dirname, '../package.json');
const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
const VERSION = pkg.version;
const REPO = 'rolfmadsen/xGauntlet';

function getPlatformInfo() {
  const platform = os.platform();
  const arch = os.arch();
  const isWindows = platform === 'win32';
  const binName = isWindows ? 'xgauntlet.exe' : 'xgauntlet';

  let archiveName = null;
  if (platform === 'linux' && arch === 'x64') {
    archiveName = 'xgauntlet-linux-x64.tar.gz';
  } else if (platform === 'linux' && arch === 'arm64') {
    archiveName = 'xgauntlet-linux-arm64.tar.gz';
  } else if (platform === 'darwin' && arch === 'arm64') {
    archiveName = 'xgauntlet-darwin-arm64.tar.gz';
  } else if (platform === 'darwin' && arch === 'x64') {
    archiveName = 'xgauntlet-darwin-x64.tar.gz';
  } else if (platform === 'win32' && arch === 'x64') {
    archiveName = 'xgauntlet-win32-x64.zip';
  }

  return { platform, arch, isWindows, binName, archiveName };
}

function getCacheDir() {
  const { isWindows } = getPlatformInfo();
  if (isWindows) {
    const base = process.env.LOCALAPPDATA || path.join(os.homedir(), 'AppData', 'Local');
    return path.join(base, 'xgauntlet', `v${VERSION}`);
  }
  const base = process.env.XDG_CACHE_HOME || path.join(os.homedir(), '.cache');
  return path.join(base, 'xgauntlet', `v${VERSION}`);
}

function findLocalBinary() {
  const { isWindows, binName } = getPlatformInfo();

  // 1. Check Cargo target directory in local workspace
  const devCandidates = [
    path.resolve(__dirname, `../../../target/release/${binName}`),
    path.resolve(__dirname, `../../../target/debug/${binName}`),
  ];

  for (const candidate of devCandidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  // 2. Check cached binary in ~/.cache/xgauntlet/vX.Y.Z/
  const cacheDir = getCacheDir();
  const cachedBin = path.join(cacheDir, binName);
  if (fs.existsSync(cachedBin)) {
    return cachedBin;
  }

  // 3. Check system PATH (making sure it's not this launcher)
  try {
    const checkCmd = isWindows ? 'where' : 'which';
    const result = spawnSync(checkCmd, ['xgauntlet'], { encoding: 'utf8' });
    if (result.status === 0 && result.stdout) {
      const firstLine = result.stdout.trim().split(/\r?\n/)[0];
      if (firstLine && fs.existsSync(firstLine)) {
        const real = fs.realpathSync(firstLine);
        if (real !== fs.realpathSync(__filename)) {
          return firstLine;
        }
      }
    }
  } catch {
    // Ignore error
  }

  return null;
}

async function downloadAndCacheBinary() {
  const { platform, arch, isWindows, binName, archiveName } = getPlatformInfo();

  if (!archiveName) {
    throw new Error(`Unsupported platform or architecture: ${platform}-${arch}`);
  }

  const cacheDir = getCacheDir();
  const targetBin = path.join(cacheDir, binName);

  if (fs.existsSync(targetBin)) {
    return targetBin;
  }

  const downloadUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/${archiveName}`;
  process.stderr.write(`[xgauntlet] Fetching native engine v${VERSION} for ${platform}-${arch}...\n`);

  fs.mkdirSync(cacheDir, { recursive: true });
  const tempArchive = path.join(cacheDir, `temp-${Date.now()}-${archiveName}`);

  try {
    const res = await fetch(downloadUrl, { redirect: 'follow' });
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}: ${res.statusText}`);
    }

    const buffer = Buffer.from(await res.arrayBuffer());
    fs.writeFileSync(tempArchive, buffer);

    if (archiveName.endsWith('.tar.gz')) {
      execFileSync('tar', ['-xzf', tempArchive, '-C', cacheDir], { stdio: 'pipe' });
    } else if (archiveName.endsWith('.zip')) {
      try {
        execFileSync('tar', ['-xf', tempArchive, '-C', cacheDir], { stdio: 'pipe' });
      } catch {
        execFileSync('powershell', ['-NoProfile', '-Command', `Expand-Archive -Path '${tempArchive}' -DestinationPath '${cacheDir}' -Force`], { stdio: 'pipe' });
      }
    }

    if (fs.existsSync(tempArchive)) {
      fs.unlinkSync(tempArchive);
    }

    if (!isWindows && fs.existsSync(targetBin)) {
      fs.chmodSync(targetBin, 0o755);
    }

    if (fs.existsSync(targetBin)) {
      process.stderr.write(`[xgauntlet] Native engine v${VERSION} ready.\n`);
      return targetBin;
    } else {
      throw new Error(`Extracted archive did not contain expected binary: ${binName}`);
    }
  } catch (err) {
    if (fs.existsSync(tempArchive)) {
      try { fs.unlinkSync(tempArchive); } catch {}
    }
    throw err;
  }
}

async function main() {
  const args = process.argv.slice(2);
  let bin = findLocalBinary();

  if (!bin) {
    try {
      bin = await downloadAndCacheBinary();
    } catch (err) {
      const { platform, arch } = getPlatformInfo();
      if (args[0] === 'status' || args[0] === 'doctor') {
        console.log('[xgauntlet] JavaScript Bootstrapper:');
        console.log(`  Platform: ${platform} (${arch})`);
        console.log(`  Version:  v${VERSION}`);
        console.log(`  Engine:   NOT_FOUND (${err.message})`);
        console.log('  Notice:   Run `cargo build --release` in xGauntlet repo or download v0.1.0 release.');
        process.exit(0);
      }

      console.error(`\x1b[31m[xgauntlet] Error: Could not resolve native engine binary for ${platform}-${arch}.\x1b[0m`);
      console.error(`  Details: ${err.message}`);
      console.error(`  Tip: You can build it locally with: cargo build --release`);
      console.error(`       Or install directly via Cargo: cargo install --path crates/xgauntlet-cli`);
      process.exit(1);
    }
  }

  const child = spawn(bin, args, { stdio: 'inherit' });
  child.on('exit', (code, signal) => {
    if (signal) {
      process.kill(process.pid, signal);
    } else {
      process.exit(code ?? 0);
    }
  });
}

main();
