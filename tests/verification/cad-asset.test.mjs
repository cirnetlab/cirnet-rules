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

function writePositions(binary) {
  [0, 0, 0, 1, 0, 0, 0, 1, 0].forEach((value, index) => binary.writeFloatLE(value, 4 * index));
}

function writeTriangle(binary) {
  writePositions(binary);
  [0, 1, 2].forEach((value, index) => binary.writeUInt16LE(value, 36 + 2 * index));
}

function candidate(edit = () => {}, write = writeTriangle, binaryBytes = 44) {
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
  const binary = Buffer.alloc(binaryBytes);
  write(binary);
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

function interleaved(withBadNormal = false) {
  return candidate(doc => {
    doc.buffers[0].byteLength = 78;
    doc.bufferViews[0] = { buffer: 0, byteLength: 72, byteStride: 24 };
    doc.bufferViews[1].byteOffset = 72;
    doc.accessors[2] = { ...doc.accessors[0], byteOffset: 12, min: [0, 0, 1], max: [0, 0, 1] };
    doc.meshes[0].primitives[0].attributes.NORMAL = 2;
  }, binary => {
    [[0, 0], [1, 0], [0, 1]].forEach(([x, y], i) => {
      binary.writeFloatLE(x, i * 24);
      binary.writeFloatLE(y, i * 24 + 4);
      binary.writeFloatLE(withBadNormal && i === 2 ? -Infinity : 1, i * 24 + 20);
      binary.writeUInt16LE(i, 72 + i * 2);
    });
  }, 80);
}

test('checks finite UV values without interpreting optional texture bounds', t => {
  const { run } = fixture(t);
  const make = bad => candidate(doc => {
    doc.buffers[0].byteLength = 68;
    doc.bufferViews.push({ buffer: 0, byteOffset: 44, byteLength: 24 });
    doc.accessors.push({ bufferView: 2, componentType: 5126, count: 3, type: 'VEC2', min: [0, 0.1], max: [1, 1] });
    doc.meshes[0].primitives[0].attributes.TEXCOORD_0 = 2;
  }, binary => {
    writeTriangle(binary);
    binary.writeFloatLE(bad ? Infinity : 0, 48);
  }, 68);
  assert.equal(run(make(false)).status, 'PASS');
  assert.equal(run(make(true)).status, 'FAIL');
});

test('checks unit quaternions on nodes in the active scene', async t => {
  const { run } = fixture(t);
  const make = rotation => candidate(doc => { doc.nodes[0].rotation = rotation; });
  const floatComponent = Math.fround(Math.SQRT1_2);
  for (const rotation of [
    [0, 0, 0, 1],
    [0, 0, -1, 0],
    [floatComponent, 0, 0, floatComponent],
    [0.259, 0, 0, 0.966],
  ]) assert.equal(run(make(rotation)).status, 'PASS');
  for (const [name, rotation] of [
    ['zero quaternion', [0, 0, 0, 0]],
    ['length below the rounding tolerance', [0, 0, 0, 0.9989]],
    ['length above the rounding tolerance', [0.708, 0, 0, 0.708]],
    ['component above one with a near-unit length', [1.0001, 0, 0, 0]],
    ['component below minus one with a near-unit length', [-1.0001, 0, 0, 0]],
  ]) {
    await t.test(name, () => {
      const { status, reason } = run(make(rotation));
      assert.deepEqual({ status, reason }, { status: 'FAIL', reason: 'invalid-node-rotation' });
    });
  }
  assert.equal(run(make([0, '0', 0, 1])).reason, 'invalid-node-transform');
});

test('checks optional node name types in the active scene', t => {
  const { run } = fixture(t);
  const make = node => candidate(doc => {
    doc.nodes.push(node);
    doc.scenes[0].nodes.push(2);
  });
  assert.equal(run(make({})).status, 'PASS');
  assert.equal(run(make({ name: '' })).status, 'PASS');
  const { status, reason } = run(make({ name: 42 }));
  assert.deepEqual({ status, reason }, { status: 'FAIL', reason: 'invalid-node-name' });
});

test('checks core declarations on materials used by active primitives', async t => {
  const { run } = fixture(t);
  const make = (material, pbr = {}) => candidate(doc => {
    doc.materials = [{ ...material, pbrMetallicRoughness: pbr }];
    doc.meshes[0].primitives[0].material = 0;
  });
  const factors = { baseColorFactor: [0, 1, 0.5, 1], metallicFactor: 1, roughnessFactor: 0 };
  for (const alphaMode of ['OPAQUE', 'MASK', 'BLEND']) {
    assert.equal(run(make({ alphaMode, alphaCutoff: 1.5, emissiveFactor: [1, 0, 0.5], doubleSided: true }, factors)).status,
      'PASS');
  }
  assert.equal(run(make({ alphaMode: 'MASK' })).status, 'PASS');
  for (const [name, material, pbr] of [
    ['short base color factor', {}, { baseColorFactor: [0, 1, 0] }],
    ['nonnumeric base color component', {}, { baseColorFactor: [0, '1', 0, 1] }],
    ['out-of-range base color component', {}, { baseColorFactor: [0, 1.1, 0, 1] }],
    ['out-of-range metallic factor', {}, { metallicFactor: 1.1 }],
    ['nonnumeric metallic factor', {}, { metallicFactor: '0.5' }],
    ['negative roughness factor', {}, { roughnessFactor: -0.1 }],
    ['nonnumeric roughness factor', {}, { roughnessFactor: null }],
    ['long emissive factor', { emissiveFactor: [0, 0, 0, 0] }],
    ['out-of-range emissive component', { emissiveFactor: [-0.1, 0, 0] }],
    ['unknown alpha mode', { alphaMode: 'ADD' }],
    ['nonnumeric alpha cutoff', { alphaMode: 'MASK', alphaCutoff: '0.5' }],
    ['negative alpha cutoff', { alphaMode: 'MASK', alphaCutoff: -0.1 }],
    ['alpha cutoff without an explicit mode', { alphaCutoff: 0.5 }],
    ['nonboolean double-sided declaration', { doubleSided: 1 }],
  ]) await t.test(name, () => assert.equal(run(make(material, pbr)).status, 'FAIL'));
});

test('checks camera references in the active scene', async t => {
  const { run } = fixture(t);
  const camera = { type: 'perspective', perspective: { yfov: 0.7, znear: 0.1 } };
  const make = (index, cameras = [camera]) => candidate(doc => {
    if (cameras !== null) doc.cameras = cameras;
    doc.nodes[1].camera = index;
  });
  for (const [name, index, cameras] of [
    ['null camera index', null],
    ['fractional camera index', 0.5],
    ['negative camera index', -1],
    ['camera index beyond the array', 1],
    ['camera array missing', 0, null],
    ['camera entry is not an object', 0, [null]],
  ]) {
    await t.test(name, () => {
      const { status, reason } = run(make(index, cameras));
      assert.deepEqual({ status, reason }, { status: 'FAIL', reason: 'invalid-gltf-reference' });
    });
  }
  await t.test('active camera is unsupported', () => {
    const { status, reason } = run(make(0));
    assert.deepEqual({ status, reason }, { status: 'BLOCKED', reason: 'unsupported-camera' });
  });
  await t.test('camera in an inactive scene is outside the intake', () => {
    assert.equal(run(candidate(doc => {
      doc.cameras = [camera];
      doc.nodes.push({ camera: 0 });
      doc.scenes.push({ nodes: [2] });
    })).status, 'PASS');
  });
});

test('checks active material texture bindings and embedded image signatures', async t => {
  const { run } = fixture(t);
  const png = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+jP1sAAAAASUVORK5CYII=', 'base64');
  const make = (edit = () => {}, image = png) => candidate(doc => {
    doc.buffers[0].byteLength = 68 + image.length;
    doc.bufferViews.push({ buffer: 0, byteOffset: 44, byteLength: 24 },
      { buffer: 0, byteOffset: 68, byteLength: image.length });
    doc.accessors.push({ bufferView: 2, componentType: 5126, count: 3, type: 'VEC2' });
    doc.images = [{ bufferView: 3, mimeType: 'image/png' }];
    doc.samplers = [{}];
    doc.textures = [{ source: 0, sampler: 0 }];
    doc.materials = [{ pbrMetallicRoughness: { baseColorTexture: { index: 0 } } }];
    const primitive = doc.meshes[0].primitives[0];
    primitive.attributes.TEXCOORD_0 = 2;
    primitive.material = 0;
    edit(doc, primitive);
  }, binary => {
    writeTriangle(binary);
    image.copy(binary, 68);
  }, Math.ceil((68 + image.length) / 4) * 4);
  assert.equal(run(make()).status, 'PASS');
  assert.equal(run(make(doc => {
    doc.materials[0].normalTexture = { index: 0, scale: -2 };
    doc.materials[0].occlusionTexture = { index: 0, strength: 0 };
  })).status, 'PASS');
  assert.equal(run(make((doc, primitive) => {
    primitive.attributes.TEXCOORD_1 = 2;
    doc.materials[0].pbrMetallicRoughness.baseColorTexture.texCoord = 1;
    delete doc.textures[0].sampler;
  })).status, 'PASS');
  assert.equal(run(make(doc => { doc.images[0].mimeType = 'image/jpeg'; },
    Buffer.from([0xff, 0xd8, 0xff, 0xe0]))).status, 'PASS');
  for (const [name, edit] of Object.entries({
    'missing texture': doc => { doc.materials[0].pbrMetallicRoughness.baseColorTexture.index = 5; },
    'missing source image': doc => { doc.textures[0].source = 5; },
    'missing sampler': doc => { doc.textures[0].sampler = 5; },
    'invalid sampler enum': doc => { doc.samplers[0].magFilter = 9987; },
    'missing default UV': (_doc, primitive) => { delete primitive.attributes.TEXCOORD_0; },
    'missing explicit UV': doc => { doc.materials[0].normalTexture = { index: 0, texCoord: 1 }; },
    'invalid UV declaration': doc => { doc.materials[0].pbrMetallicRoughness.baseColorTexture.texCoord = null; },
    'nonnumeric normal scale': doc => { doc.materials[0].normalTexture = { index: 0, scale: '1' }; },
    'nonnumeric occlusion strength': doc => { doc.materials[0].occlusionTexture = { index: 0, strength: null }; },
    'out-of-range occlusion strength': doc => { doc.materials[0].occlusionTexture = { index: 0, strength: 1.1 }; },
    'invalid PBR declaration': doc => { doc.materials[0].pbrMetallicRoughness = null; },
    'embedded image view marked as geometry': doc => { doc.bufferViews[3].target = 34962; },
    'geometry reuses the embedded image view': doc => { doc.accessors[2].bufferView = 3; },
    'mismatched image MIME': doc => { doc.images[0].mimeType = 'image/jpeg'; },
  })) await t.test(name, () => assert.equal(run(make(edit)).status, 'FAIL'));
  assert.equal(run(make(doc => {
    doc.materials[0].pbrMetallicRoughness.baseColorTexture.extensions = { KHR_texture_transform: {} };
  })).status, 'BLOCKED');
  assert.equal(run(make(undefined, Buffer.alloc(png.length))).status, 'FAIL');
});

test('requires four-byte element alignment for normalized short vertex layouts', t => {
  const { run } = fixture(t);
  const make = padded => candidate(doc => {
    doc.buffers[0].byteLength = 56;
    doc.bufferViews.push(padded
      ? { buffer: 0, byteOffset: 44, byteLength: 12, byteStride: 4 }
      : { buffer: 0, byteOffset: 44, byteLength: 6 });
    doc.accessors.push({ bufferView: 2, componentType: 5121, count: 3, type: 'VEC2', normalized: true });
    doc.meshes[0].primitives[0].attributes.TEXCOORD_0 = 2;
  }, writeTriangle, 56);
  assert.equal(run(make(true)).status, 'PASS');
  assert.equal(run(make(false)).status, 'FAIL');
});

test('decodes little-endian triangle indices at each supported width', async t => {
  const { run } = fixture(t);
  for (const [componentType, width] of [[5121, 1], [5123, 2], [5125, 4]]) {
    await t.test(`componentType ${componentType}`, () => {
      const make = last => candidate(doc => {
        doc.buffers[0].byteLength = 36 + 3 * width;
        doc.bufferViews[1].byteLength = 3 * width;
        doc.accessors[1].componentType = componentType;
      }, binary => {
        writePositions(binary);
        [0, 1, last].forEach((value, index) => binary.writeUIntLE(value, 36 + width * index, width));
      }, Math.ceil((36 + 3 * width) / 4) * 4);
      assert.equal(run(make(2)).status, 'PASS');
      assert.equal(run(make(3)).status, 'FAIL');
    });
  }
});

test('rejects the reserved maximum index inside a large enough vertex accessor', t => {
  const { run } = fixture(t);
  const make = last => candidate(doc => {
    doc.buffers[0].byteLength = 3075;
    doc.bufferViews = [{ buffer: 0, byteLength: 3072 }, { buffer: 0, byteOffset: 3072, byteLength: 3 }];
    doc.accessors[0] = { ...doc.accessors[0], count: 256, max: [0, 0, 0] };
    doc.accessors[1].componentType = 5121;
  }, binary => { binary[3074] = last; }, 3076);
  assert.equal(run(make(254)).status, 'PASS');
  assert.equal(run(make(255)).status, 'FAIL');
});

test('rejects non-finite vertex components while accepting valid interleaving', async t => {
  const { run } = fixture(t);
  assert.equal(run(interleaved()).status, 'PASS');
  for (const [name, glb] of [
    ['NaN POSITION', candidate(undefined, binary => { writeTriangle(binary); binary.writeFloatLE(NaN); })],
    ['infinite NORMAL', interleaved(true)],
  ]) await t.test(name, () => assert.equal(run(glb).status, 'FAIL'));
});

test('rejects inconsistent supported accessor declarations', async t => {
  const { run } = fixture(t);
  for (const [name, edit] of Object.entries({
    'missing position bounds': doc => { delete doc.accessors[0].min; },
    'bounds differ from binary': doc => { doc.accessors[0].max = [1, 2, 0]; },
    'index bounds differ from binary': doc => { doc.accessors[1].max = [1]; },
    'normalized floating point': doc => { doc.accessors[0].normalized = true; },
    'invalid attribute reference': doc => { doc.meshes[0].primitives[0].attributes.TEXCOORD_0 = 99; },
    'wrong texture coordinate layout': doc => { doc.meshes[0].primitives[0].attributes.TEXCOORD_0 = 0; },
    'vertex view targets indices': doc => { doc.bufferViews[0].target = 34963; },
    'two vertex accessors in an unstrided view': doc => {
      doc.accessors.push({ ...doc.accessors[0] });
      doc.meshes[0].primitives[0].attributes.NORMAL = 2;
    },
    'index and vertex accessors share a view': doc => { doc.accessors[1].bufferView = 0; },
    'aligning relative but not absolute index offset': doc => {
      doc.bufferViews[1].byteOffset = 35;
    },
  })) await t.test(name, () => assert.equal(run(candidate(edit)).status, 'FAIL'));
});

test('blocks excessive scan work from distinct accessors sharing the same bytes', t => {
  const { run } = fixture(t);
  const vertices = 262146;
  const bytes = 12 * vertices;
  const glb = candidate(doc => {
    doc.buffers[0].byteLength = bytes;
    doc.bufferViews = [{ buffer: 0, byteLength: bytes, byteStride: 12 }];
    doc.accessors = [{ ...doc.accessors[0], count: vertices, max: [0, 0, 0] }];
    const primitive = doc.meshes[0].primitives[0];
    delete primitive.indices;
    for (let i = 1; i <= 10; i++) {
      doc.accessors.push({ ...doc.accessors[0] });
      primitive.attributes[`_ALIAS_${i}`] = i;
    }
  }, () => {}, bytes);
  const result = run(glb, { budgets: { bytes: glb.length, triangles: 2 * vertices / 3, primitiveInstances: 2 } });
  assert.equal(result.status, 'BLOCKED');
  assert.equal(result.reason, 'geometry-scan-limit');
});

function unlit(doc) {
  doc.extensionsUsed = ['KHR_materials_unlit'];
  doc.extensionsRequired = ['KHR_materials_unlit'];
  doc.materials = [{ extensions: { KHR_materials_unlit: {} } }];
  doc.meshes[0].primitives[0].material = 0;
}

test('accepts supported unlit declarations and distinguishes malformed and unsupported extensions', async t => {
  const { run } = fixture(t);
  for (const [name, expected, edit] of [
    ['required unlit', 'PASS', unlit],
    ['optional unlit', 'PASS', doc => { unlit(doc); delete doc.extensionsRequired; }],
    ['required without used', 'FAIL', doc => { unlit(doc); delete doc.extensionsUsed; }],
    ['required without material binding', 'FAIL', doc => { unlit(doc); delete doc.materials[0].extensions; }],
    ['invalid unlit material record', 'FAIL', doc => { unlit(doc); doc.materials[0].extensions.KHR_materials_unlit = null; }],
    ['duplicate declaration', 'FAIL', doc => { unlit(doc); doc.extensionsUsed.push('KHR_materials_unlit'); }],
    ['unlisted material extension', 'BLOCKED', doc => { unlit(doc); doc.materials[0].extensions.KHR_materials_clearcoat = {}; }],
    ['mixed required extensions', 'BLOCKED', doc => { unlit(doc); doc.extensionsRequired.push('KHR_draco_mesh_compression'); }],
  ]) await t.test(name, () => assert.equal(run(candidate(edit)).status, expected));
});
