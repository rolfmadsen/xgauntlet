#!/usr/bin/env node
/**
 * xgauntlet - Universal cross-platform verification gauntlet for AI coding agents.
 * Bootstrapper that resolves, caches, or downloads the native Rust engine binary.
 */

import { spawnSync, spawn, execFileSync } from 'node:child_process';
import path from 'node:path';
import fs from 'node:fs';
import os from 'node:os';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const pkgJsonPath = path.resolve(__dirname, '../package.json');
const pkg = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
const VERSION = pkg.version;
const REPO = 'rolfmadsen/xgauntlet';

export const EXPECTED_REPOSITORY_ID = '1359184794';
export const EXPECTED_REPO = 'rolfmadsen/xgauntlet';
export const EXPECTED_WORKFLOW = '.github/workflows/release.yml';

export function extractBundleClaims(bundle, statement) {
  let repositoryId = null;
  let repository = null;
  let workflow = null;
  let ref = null;

  if (statement?.predicate) {
    const pred = statement.predicate;
    repositoryId =
      pred.buildDefinition?.internalParameters?.github?.repository_id ||
      pred.invocation?.environment?.github?.repository_id ||
      null;

    repository =
      pred.buildDefinition?.externalParameters?.workflow?.repository ||
      pred.invocation?.configSource?.uri ||
      null;

    workflow =
      pred.buildDefinition?.externalParameters?.workflow?.path ||
      pred.invocation?.configSource?.entryPoint ||
      null;

    ref =
      pred.buildDefinition?.externalParameters?.workflow?.ref ||
      pred.invocation?.configSource?.ref ||
      null;
  }

  const certBytes =
    bundle?.verificationMaterial?.certificate?.rawBytes ||
    bundle?.verificationMaterial?.x509CertificateChain?.certificates?.[0]?.rawBytes;

  if (certBytes) {
    const certBuf = Buffer.isBuffer(certBytes) ? certBytes : Buffer.from(certBytes, 'base64');
    const certStr = certBuf.toString('utf8');

    if (!repositoryId && certStr.includes('1359184794')) {
      repositoryId = '1359184794';
    }
    if (!repository && certStr.includes('rolfmadsen/xgauntlet')) {
      repository = 'rolfmadsen/xgauntlet';
    }
    if (!workflow && certStr.includes('.github/workflows/release.yml')) {
      workflow = '.github/workflows/release.yml';
    }
  }

  return { repositoryId, repository, workflow, ref };
}

export async function verifyReleaseArchive({
  archiveBuffer,
  archiveName,
  bundle,
  expectedRepoId = EXPECTED_REPOSITORY_ID,
  expectedRepo = EXPECTED_REPO,
  expectedWorkflow = EXPECTED_WORKFLOW,
  expectedRef,
  sigstoreVerifier,
  claimsExtractor,
}) {
  if (!bundle) {
    throw new Error('INVALID_BUNDLE: Missing attestation bundle');
  }

  if (sigstoreVerifier) {
    await sigstoreVerifier(bundle, archiveBuffer);
  } else {
    try {
      const sigstore = await import('sigstore');
      await sigstore.verify(bundle, archiveBuffer);
    } catch (err) {
      throw new Error(`SIGNATURE_VERIFICATION_FAILED: ${err.message}`);
    }
  }

  if (!bundle.dsseEnvelope?.payload) {
    throw new Error('INVALID_BUNDLE: Missing dsseEnvelope payload in attestation bundle');
  }

  const payloadStr = Buffer.from(bundle.dsseEnvelope.payload, 'base64').toString('utf8');
  let statement;
  try {
    statement = JSON.parse(payloadStr);
  } catch (err) {
    throw new Error(`INVALID_PAYLOAD: Failed to parse in-toto statement JSON: ${err.message}`);
  }

  const actualDigest = crypto.createHash('sha256').update(archiveBuffer).digest('hex');

  const subjects = statement.subject || [];
  const matchingSubject = subjects.find((s) => {
    const subjDigest = s.digest?.sha256;
    if (!subjDigest) return false;
    return subjDigest.toLowerCase() === actualDigest.toLowerCase();
  });

  if (!matchingSubject) {
    throw new Error(
      `DIGEST_MISMATCH: Archive digest (${actualDigest}) does not match any attested subject in bundle`
    );
  }

  const claims = claimsExtractor
    ? claimsExtractor(bundle, statement)
    : extractBundleClaims(bundle, statement);

  if (claims.repositoryId && String(claims.repositoryId) !== String(expectedRepoId)) {
    throw new Error(
      `REPOSITORY_ID_MISMATCH: Attested repository ID '${claims.repositoryId}' does not match expected ID '${expectedRepoId}'`
    );
  }

  if (claims.repository && !claims.repository.toLowerCase().includes(expectedRepo.toLowerCase())) {
    throw new Error(
      `REPOSITORY_MISMATCH: Attested repository '${claims.repository}' does not match expected repository '${expectedRepo}'`
    );
  }

  if (claims.workflow && !claims.workflow.includes(expectedWorkflow)) {
    throw new Error(
      `WORKFLOW_MISMATCH: Attested workflow '${claims.workflow}' does not match expected workflow '${expectedWorkflow}'`
    );
  }

  if (expectedRef && claims.ref && claims.ref !== expectedRef) {
    throw new Error(
      `REF_MISMATCH: Attested ref '${claims.ref}' does not match expected ref '${expectedRef}'`
    );
  }

  return {
    verified: true,
    digest: actualDigest,
    claims,
  };
}

export function getPlatformInfo(platform = os.platform(), arch = os.arch()) {
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
  if (process.env.XGAUNTLET_TEST_FORCE_NOT_FOUND === '1') {
    return null;
  }
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

export async function fetchAttestationBundle(repo = REPO, version = VERSION, customFetcher = null) {
  const fetchFn = customFetcher || fetch;
  const candidates = [
    `https://github.com/${repo}/releases/download/v${version}/attestation.bundle`,
    `https://github.com/${repo}/releases/download/v${version}/attestation.json`,
  ];

  let lastStatus = 404;
  let lastStatusText = 'Not Found';

  for (const url of candidates) {
    try {
      const res = await fetchFn(url, { redirect: 'follow' });
      if (res && res.ok) {
        return await res.json();
      }
      if (res) {
        lastStatus = res.status;
        lastStatusText = res.statusText || 'Not Found';
      }
    } catch (err) {
      lastStatusText = err.message;
    }
  }

  throw new Error(`HTTP ${lastStatus} fetching attestation bundle: ${lastStatusText}`);
}

async function downloadAndCacheBinary() {
  if (process.env.XGAUNTLET_TEST_FORCE_NOT_FOUND === '1') {
    throw new Error('Forced missing binary for test');
  }
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

  let tempArchive = null;
  let createdCacheDir = false;

  try {
    const [resArchive, bundleJson] = await Promise.all([
      fetch(downloadUrl, { redirect: 'follow' }),
      fetchAttestationBundle(REPO, VERSION),
    ]);

    if (!resArchive.ok) {
      throw new Error(`HTTP ${resArchive.status} fetching binary archive: ${resArchive.statusText}`);
    }

    const archiveBuffer = Buffer.from(await resArchive.arrayBuffer());

    process.stderr.write(`[xgauntlet] Verifying cryptographic release attestation for ${archiveName}...\n`);
    await verifyReleaseArchive({
      archiveBuffer,
      archiveName,
      bundle: bundleJson,
      expectedRepoId: EXPECTED_REPOSITORY_ID,
      expectedRepo: EXPECTED_REPO,
      expectedWorkflow: EXPECTED_WORKFLOW,
      expectedRef: `refs/tags/v${VERSION}`,
    });
    process.stderr.write(`[xgauntlet] Attestation verified successfully (Repo ID: ${EXPECTED_REPOSITORY_ID}).\n`);

    if (!fs.existsSync(cacheDir)) {
      fs.mkdirSync(cacheDir, { recursive: true });
      createdCacheDir = true;
    }
    tempArchive = path.join(cacheDir, `temp-${Date.now()}-${archiveName}`);
    fs.writeFileSync(tempArchive, archiveBuffer);

    if (archiveName.endsWith('.tar.gz')) {
      execFileSync('tar', ['-xzf', tempArchive, '-C', cacheDir], { stdio: 'pipe' });
    } else if (archiveName.endsWith('.zip')) {
      try {
        execFileSync('tar', ['-xf', tempArchive, '-C', cacheDir], { stdio: 'pipe' });
      } catch {
        execFileSync('powershell', ['-NoProfile', '-Command', `Expand-Archive -Path '${tempArchive}' -DestinationPath '${cacheDir}' -Force`], { stdio: 'pipe' });
      }
    }

    if (tempArchive && fs.existsSync(tempArchive)) {
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
    if (tempArchive && fs.existsSync(tempArchive)) {
      try { fs.unlinkSync(tempArchive); } catch {}
    }
    if (createdCacheDir && fs.existsSync(cacheDir)) {
      try {
        const files = fs.readdirSync(cacheDir);
        if (files.length === 0) {
          fs.rmdirSync(cacheDir);
        }
      } catch {}
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
        console.log(`  Notice:   Run \`cargo build --release\` in xGauntlet repo or download v${VERSION} release.`);
        process.exit(1);
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

function isMainModule() {
  if (!process.argv[1]) return false;
  try {
    const scriptPath = fs.realpathSync(process.argv[1]);
    const modulePath = fs.realpathSync(fileURLToPath(import.meta.url));
    return scriptPath === modulePath;
  } catch {
    return false;
  }
}

if (isMainModule()) {
  main();
}
