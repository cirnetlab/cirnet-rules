import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtempSync, realpathSync, rmSync, writeFileSync, linkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';

const cli = fileURLToPath(new URL('../../scripts/verify-cad-asset.mjs', import.meta.url));
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');

function candidate(edit = () => {}) {
  const doc = {
    asset: { version: '2.0' },
    scene: 0, scenes: [{ nodes: [0, 1] }],
    nodes: [{ name: 'enclosure', mesh: 0 }, { name: 'connector-datum', mesh: 0 }],
    meshes: [{ primitives: [{ attributes: { POSITION: 0 }, indices: 1 }] }],
    buffers: [{ byteLength: 42 }],
    bufferViews: [{ buffer: 0, byteLength: 36 }, { buffer: 0, byteOffset: 36, byteLength: 6 }],
    accessors: [
      { bufferView: 0, componentType: 5126, count: 3, type: 'VEC3', min: [0, 0, 0], max: [1, 1, 0] },
      { bufferView: 1, componentType: 5123, count: 3, type: 'SCALAR' },
    ],
  };
  edit(doc);
  const json = Buffer.from(JSON.stringify(doc));
  const jsonSize = Math.ceil(json.length / 4) * 4;
  const binary = Buffer.alloc(44);
  [0, 0, 0, 1, 0, 0, 0, 1, 0].forEach((value, index) => binary.writeFloatLE(value, 4 * index));
  [0, 1, 2].forEach((value, index) => binary.writeUInt16LE(value, 36 + 2 * index));
  const glb = Buffer.alloc(28 + jsonSize + binary.length);
  glb.write('glTF');
  glb.writeUInt32LE(2, 4);
  glb.writeUInt32LE(glb.length, 8);
  glb.writeUInt32LE(jsonSize, 12);
  glb.write('JSON', 16);
  glb.fill(32, 20, 20 + jsonSize);
  json.copy(glb, 20);
  glb.writeUInt32LE(binary.length, 20 + jsonSize);
  glb.writeUInt32LE(0x004e4942, 24 + jsonSize);
  binary.copy(glb, 28 + jsonSize);
  return glb;
}

function fixture(t) {
  const temp = realpathSync(tmpdir());
  const root = mkdtempSync(join(temp, 'cirnet-cad-asset-'));
  t.after(() => {
    assert.equal(realpathSync(root), root);
    assert.equal(dirname(root), temp);
    assert.ok(basename(root).startsWith('cirnet-cad-asset-'));
    rmSync(root, { recursive: true, force: true });
  });
  const write = (file, bytes) => writeFileSync(join(root, file), bytes);
  const run = (glb, overrides = {}, options = {}) => {
    const expectation = JSON.stringify({
      schemaVersion: 1, path: 'enclosure.glb', sha256: sha256(glb),
      budgets: { bytes: 4096, triangles: 2, primitiveInstances: 2 },
      requiredNodes: ['enclosure', 'connector-datum'], ...overrides,
    });
    if (!options.unwritten) write('enclosure.glb', glb);
    write('expected.json', expectation);
    const result = spawnSync(process.execPath, [
      cli, '--root', root, '--expectation', 'expected.json',
      '--sha256', options.pin ?? sha256(expectation),
    ], { encoding: 'utf8', timeout: 10_000 });
    assert.ifError(result.error);
    assert.equal(result.stderr, '');
    const report = JSON.parse(result.stdout);
    assert.equal(result.status, { PASS: 0, FAIL: 1, BLOCKED: 3 }[report.status]);
    return report;
  };
  return { root, write, run };
}

test('assesses a pinned embedded GLB and counts both uses of one mesh', t => {
  const glb = candidate();
  const result = fixture(t).run(glb);
  assert.equal(result.status, 'PASS');
  assert.equal(result.scope, 'static-glb-intake');
  assert.deepEqual(result.metrics, { sha256: sha256(glb), bytes: glb.length, triangles: 2, meshNodes: 2, primitiveInstances: 2 });
});

test('rejects stale pins, traversal, missing files and linked input without disclosure', t => {
  const { root, write, run } = fixture(t);
  const glb = candidate();
  assert.equal(run(glb, {}, { unwritten: true }).status, 'BLOCKED');
  assert.equal(run(glb, {}, { pin: '0'.repeat(64) }).status, 'BLOCKED');
  assert.equal(run(glb, { path: '../private.glb' }).status, 'BLOCKED');
  assert.equal(run(glb, { sha256: '0'.repeat(64) }).status, 'FAIL');
  write('private.glb', glb);
  linkSync(join(root, 'private.glb'), join(root, 'linked.glb'));
  assert.equal(run(glb, { path: 'linked.glb' }).status, 'BLOCKED');
});

test('fails budgets, missing names and ambiguous or unreachable named components', t => {
  const { run } = fixture(t);
  for (const budgets of [
    { bytes: 1, triangles: 2, primitiveInstances: 2 },
    { bytes: 4096, triangles: 1, primitiveInstances: 2 },
    { bytes: 4096, triangles: 2, primitiveInstances: 1 },
  ]) assert.equal(run(candidate(), { budgets }).status, 'FAIL');
  for (const edit of [
    doc => { doc.nodes[1].name = 'other'; },
    doc => { doc.scenes[0].nodes = [0]; },
    doc => { doc.nodes.push({ name: 'connector-datum' }); doc.scenes[0].nodes.push(2); },
  ]) assert.equal(run(candidate(edit)).status, 'FAIL');
});

test('fails malformed chunks, unsupported dependencies and inconsistent geometry declarations', t => {
  const { run } = fixture(t);
  assert.equal(run(candidate().subarray(0, -1)).status, 'FAIL');
  const invalidMagic = candidate();
  invalidMagic[0] |= 0x80;
  assert.equal(run(invalidMagic).status, 'FAIL');
  for (const edit of [
    doc => { doc.buffers[0].uri = 'private.bin'; },
    doc => { doc.images = [{ uri: 'private.png' }]; },
    doc => { doc.nodes[0].children = [0]; },
    doc => { doc.nodes[1].matrix = [1, 0, 0]; },
    doc => { doc.scene = null; },
    doc => { doc.meshes[0].primitives[0].mode = null; },
    doc => { doc.bufferViews[1].byteOffset = 40; },
    doc => { doc.bufferViews[0].byteStride = null; },
    doc => { doc.accessors[0].byteOffset = null; },
    doc => { doc.accessors[1].count = 6; },
    doc => { doc.meshes[0].primitives[0].attributes.NORMAL = 1; },
  ]) assert.equal(run(candidate(edit)).status, 'FAIL');
  for (const edit of [
    doc => { doc.extensionsRequired = ['KHR_draco_mesh_compression']; },
    doc => { doc.extensionsUsed = ['EXT_mesh_gpu_instancing']; },
    doc => { doc.meshes[0].primitives[0].mode = 5; },
    doc => { doc.bufferViews[0].extensions = { EXT_meshopt_compression: {} }; },
  ]) assert.equal(run(candidate(edit)).status, 'BLOCKED');
});
