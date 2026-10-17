import { test } from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import fs from 'node:fs';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const releaseYamlPath = path.resolve(__dirname, '../../../.github/workflows/release.yml');
const binPath = path.resolve(__dirname, '../bin/xgauntlet.js');

test('AC1: release.yml has dedicated attest-binaries job with tight permissions and SHA-pinned actions', () => {
  const content = fs.readFileSync(releaseYamlPath, 'utf8');

  // Must have attest-binaries job
  assert.match(content, /attest-binaries:/, 'Must contain attest-binaries job');

  // Must have narrow permissions in attest-binaries
  assert.match(content, /id-token:\s*write/, 'Must grant id-token: write');
  assert.match(content, /attestations:\s*write/, 'Must grant attestations: write');
  assert.match(content, /artifact-metadata:\s*write/, 'Must grant artifact-metadata: write');
  assert.match(content, /contents:\s*read/, 'Must limit contents to read in attest-binaries');

  // Must use actions/attest pinned by full commit SHA (40 hex chars)
  assert.match(
    content,
    /uses:\s*actions\/attest@[0-9a-f]{40}/,
    'Must use actions/attest pinned to a 40-character commit SHA'
  );

  // Must use actions/upload-artifact pinned by SHA
  assert.match(
    content,
    /uses:\s*actions\/upload-artifact@[0-9a-f]{40}/,
    'Must use actions/upload-artifact pinned to a 40-character commit SHA'
  );

  // Must use actions/download-artifact pinned by SHA
  assert.match(
    content,
    /uses:\s*actions\/download-artifact@[0-9a-f]{40}/,
    'Must use actions/download-artifact pinned to a 40-character commit SHA'
  );

  // attest-binaries must depend on [gate-check-release, build-binaries]
  assert.match(
    content,
    /needs:\s*\[\s*gate-check-release,\s*build-binaries\s*\]/,
    'attest-binaries must depend on gate-check-release and build-binaries'
  );

  // create-github-release must depend on attest-binaries
  assert.match(
    content,
    /create-github-release:[\s\S]*?needs:\s*\[[^\]]*?attest-binaries[^\]]*?\]/,
    'create-github-release must require attest-binaries'
  );
});

test('AC2 & AC3: xgauntlet.js bootstrapper exports static anchored trust policy constants', async () => {
  const launcher = await import('../bin/xgauntlet.js');

  assert.equal(
    launcher.EXPECTED_REPOSITORY_ID,
    '1359184794',
    'Repository ID must be statically anchored to 1359184794'
  );
  assert.equal(
    launcher.EXPECTED_REPO,
    'rolfmadsen/xgauntlet',
    'Repository name must match rolfmadsen/xgauntlet'
  );
  assert.equal(
    launcher.EXPECTED_WORKFLOW,
    '.github/workflows/release.yml',
    'Workflow path must match .github/workflows/release.yml'
  );
  assert.equal(
    typeof launcher.verifyReleaseArchive,
    'function',
    'Must export verifyReleaseArchive function'
  );
});

test('AC2 & AC3: verifyReleaseArchive succeeds with valid bundle and matching digest', async () => {
  const { verifyReleaseArchive } = await import('../bin/xgauntlet.js');

  const archiveBuffer = Buffer.from('dummy tarball contents');
  const archiveDigest = crypto.createHash('sha256').update(archiveBuffer).digest('hex');

  const mockBundle = {
    mediaType: 'application/vnd.dev.sigstore.bundle.v0.3+json',
    dsseEnvelope: {
      payloadType: 'application/vnd.in-toto+json',
      payload: Buffer.from(
        JSON.stringify({
          _type: 'https://in-toto.io/Statement/v1',
          subject: [
            {
              name: 'xgauntlet-linux-x64.tar.gz',
              digest: { sha256: archiveDigest },
            },
          ],
          predicateType: 'https://slsa.dev/provenance/v1',
          predicate: {},
        })
      ).toString('base64'),
    },
    verificationMaterial: {
      certificate: {
        rawBytes: 'mock-cert',
      },
    },
  };

  const mockVerifier = async () => true;
  const mockExtractor = () => ({
    repositoryId: '1359184794',
    repository: 'rolfmadsen/xgauntlet',
    workflow: '.github/workflows/release.yml',
    ref: 'refs/tags/v0.8.2',
  });

  const result = await verifyReleaseArchive({
    archiveBuffer,
    archiveName: 'xgauntlet-linux-x64.tar.gz',
    bundle: mockBundle,
    sigstoreVerifier: mockVerifier,
    claimsExtractor: mockExtractor,
  });

  assert.equal(result.verified, true);
  assert.equal(result.digest, archiveDigest);
});

test('AC3: verifyReleaseArchive fails closed when archive digest does not match bundle', async () => {
  const { verifyReleaseArchive } = await import('../bin/xgauntlet.js');

  const archiveBuffer = Buffer.from('tampered tarball contents');
  const mockBundle = {
    mediaType: 'application/vnd.dev.sigstore.bundle.v0.3+json',
    dsseEnvelope: {
      payloadType: 'application/vnd.in-toto+json',
      payload: Buffer.from(
        JSON.stringify({
          _type: 'https://in-toto.io/Statement/v1',
          subject: [
            {
              name: 'xgauntlet-linux-x64.tar.gz',
              digest: { sha256: '0000000000000000000000000000000000000000000000000000000000000000' },
            },
          ],
        })
      ).toString('base64'),
    },
  };

  await assert.rejects(
    async () => {
      await verifyReleaseArchive({
        archiveBuffer,
        archiveName: 'xgauntlet-linux-x64.tar.gz',
        bundle: mockBundle,
        sigstoreVerifier: async () => true,
        claimsExtractor: () => ({
          repositoryId: '1359184794',
          repository: 'rolfmadsen/xgauntlet',
          workflow: '.github/workflows/release.yml',
        }),
      });
    },
    /DIGEST_MISMATCH|digest.*mismatch/i
  );
});

test('AC3: verifyReleaseArchive fails closed when repository ID does not match trust policy', async () => {
  const { verifyReleaseArchive } = await import('../bin/xgauntlet.js');

  const archiveBuffer = Buffer.from('dummy tarball contents');
  const archiveDigest = crypto.createHash('sha256').update(archiveBuffer).digest('hex');

  const mockBundle = {
    dsseEnvelope: {
      payloadType: 'application/vnd.in-toto+json',
      payload: Buffer.from(
        JSON.stringify({
          _type: 'https://in-toto.io/Statement/v1',
          subject: [{ name: 'xgauntlet-linux-x64.tar.gz', digest: { sha256: archiveDigest } }],
        })
      ).toString('base64'),
    },
  };

  await assert.rejects(
    async () => {
      await verifyReleaseArchive({
        archiveBuffer,
        archiveName: 'xgauntlet-linux-x64.tar.gz',
        bundle: mockBundle,
        sigstoreVerifier: async () => true,
        claimsExtractor: () => ({
          repositoryId: '999999999', // Untrusted/attacker repo ID
          repository: 'attacker/xgauntlet',
          workflow: '.github/workflows/release.yml',
        }),
      });
    },
    /REPOSITORY_ID_MISMATCH|repository.*id.*mismatch/i
  );
});

test('AC3: verifyReleaseArchive fails closed when workflow path does not match trust policy', async () => {
  const { verifyReleaseArchive } = await import('../bin/xgauntlet.js');

  const archiveBuffer = Buffer.from('dummy tarball contents');
  const archiveDigest = crypto.createHash('sha256').update(archiveBuffer).digest('hex');

  const mockBundle = {
    dsseEnvelope: {
      payloadType: 'application/vnd.in-toto+json',
      payload: Buffer.from(
        JSON.stringify({
          _type: 'https://in-toto.io/Statement/v1',
          subject: [{ name: 'xgauntlet-linux-x64.tar.gz', digest: { sha256: archiveDigest } }],
        })
      ).toString('base64'),
    },
  };

  await assert.rejects(
    async () => {
      await verifyReleaseArchive({
        archiveBuffer,
        archiveName: 'xgauntlet-linux-x64.tar.gz',
        bundle: mockBundle,
        sigstoreVerifier: async () => true,
        claimsExtractor: () => ({
          repositoryId: '1359184794',
          repository: 'rolfmadsen/xgauntlet',
          workflow: '.github/workflows/malicious.yml',
        }),
      });
    },
    /WORKFLOW_MISMATCH|workflow.*mismatch/i
  );
});

test('AC3: verifyReleaseArchive fails closed when cryptographic signature verification fails', async () => {
  const { verifyReleaseArchive } = await import('../bin/xgauntlet.js');

  const archiveBuffer = Buffer.from('dummy tarball contents');
  const archiveDigest = crypto.createHash('sha256').update(archiveBuffer).digest('hex');

  const mockBundle = {
    dsseEnvelope: {
      payload: Buffer.from(
        JSON.stringify({
          subject: [{ name: 'xgauntlet-linux-x64.tar.gz', digest: { sha256: archiveDigest } }],
        })
      ).toString('base64'),
    },
  };

  await assert.rejects(
    async () => {
      await verifyReleaseArchive({
        archiveBuffer,
        archiveName: 'xgauntlet-linux-x64.tar.gz',
        bundle: mockBundle,
        sigstoreVerifier: async () => {
          throw new Error('Signature verification failed: invalid certificate chain');
        },
        claimsExtractor: () => ({
          repositoryId: '1359184794',
          repository: 'rolfmadsen/xgauntlet',
          workflow: '.github/workflows/release.yml',
        }),
      });
    },
    /Signature verification failed/i
  );
});

test('AC1 & AC4: fetchAttestationBundle falls back to attestation.json when attestation.bundle returns 404', async () => {
  const launcher = await import('../bin/xgauntlet.js');
  assert.equal(
    typeof launcher.fetchAttestationBundle,
    'function',
    'xgauntlet.js must export fetchAttestationBundle'
  );

  const mockFetcher = async (url) => {
    if (url.endsWith('attestation.bundle')) {
      return { ok: false, status: 404, statusText: 'Not Found' };
    }
    if (url.endsWith('attestation.json')) {
      return {
        ok: true,
        status: 200,
        statusText: 'OK',
        json: async () => ({ mediaType: 'mock-bundle', fallbackSuccess: true }),
      };
    }
    return { ok: false, status: 500, statusText: 'Server Error' };
  };

  const bundle = await launcher.fetchAttestationBundle('rolfmadsen/xgauntlet', '0.8.3', mockFetcher);
  assert.equal(bundle.fallbackSuccess, true, 'Must fallback to attestation.json');
});

test('AC3 & AC4: release.yml stages both attestation.bundle and attestation.json for dual-asset parity', () => {
  const content = fs.readFileSync(releaseYamlPath, 'utf8');

  // Must stage both filenames in attest-binaries
  assert.match(
    content,
    /attestation\.bundle[\s\S]*?attestation\.json|attestation\.json[\s\S]*?attestation\.bundle/,
    'release.yml must ensure both attestation.bundle and attestation.json are uploaded'
  );
});

