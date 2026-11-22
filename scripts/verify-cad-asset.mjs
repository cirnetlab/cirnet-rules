import { createHash } from 'node:crypto';
import { closeSync, constants, fstatSync, lstatSync, openSync, readSync, realpathSync } from 'node:fs';
import { isAbsolute, join, parse, relative, resolve, sep } from 'node:path';

const maximumGlbBytes = 32 * 1024 * 1024;
const maximumExpectationBytes = 32 * 1024;
const vectorSizes = new Map([[5126, 12]]);
const indexSizes = new Map([[5121, 1], [5123, 2], [5125, 4]]);
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const isDigest = value => typeof value === 'string' && /^[a-f0-9]{64}$/u.test(value);
const isObject = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const uint = value => Number.isSafeInteger(value) && value >= 0;
const positive = value => uint(value) && value > 0;
const defined = (object, field, fallback) => Object.hasOwn(object, field) ? object[field] : fallback;
const hasFields = (value, fields) => isObject(value) && Object.keys(value).length === fields.length &&
  fields.every(field => Object.hasOwn(value, field));

class GateResult extends Error {
  constructor(status, reason) {
    super(reason);
    this.status = status;
  }
}
const blocked = reason => { throw new GateResult('BLOCKED', reason); };
const invalid = reason => { throw new GateResult('FAIL', reason); };

function argumentsFor(args) {
  const names = ['--root', '--expectation', '--sha256'];
  const options = new Map();
  for (let index = 0; index < args.length; index += 2) {
    const [key, value] = args.slice(index, index + 2);
    if (!names.includes(key) || options.has(key) || !value) blocked('invalid-arguments');
    options.set(key, value);
  }
  if (options.size !== names.length || !isDigest(options.get('--sha256'))) blocked('invalid-arguments');
  return names.map(name => options.get(name));
}

function approvedRoot(path) {
  if (!isAbsolute(path) || /^[\\/]{2}/u.test(path) || path.split(/[\\/]/u).includes('..')) blocked('invalid-root');
  const root = resolve(path);
  const volume = parse(root).root;
  if (root === volume || !lstatSync(root).isDirectory() || relative(root, realpathSync(root))) blocked('invalid-root');
  return root;
}

function readPinnedInput(root, name, limit) {
  const components = name.split('/');
  if (name.length > 256 || components.some(part => !/^[a-z0-9_.-]+$/iu.test(part) ||
      part.endsWith('.') || part.toLowerCase() === '.git' ||
      /^(?:con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/iu.test(part))) blocked('invalid-relative-path');
  let directory = root;
  for (const part of components.slice(0, -1)) {
    directory = join(directory, part);
    if (!lstatSync(directory).isDirectory() || relative(directory, realpathSync(directory))) {
      blocked('unsupported-input-directory');
    }
  }
  const path = join(directory, components.at(-1));
  const original = lstatSync(path);
  if (!original.isFile() || original.nlink !== 1 || relative(path, realpathSync(path))) blocked('unsupported-input-file');
  if (original.size > limit) blocked('input-size-limit');
  const fd = openSync(path, constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0) | (constants.O_NONBLOCK ?? 0));
  try {
    const start = fstatSync(fd);
    if (!start.isFile() || start.nlink !== 1 || start.ino !== original.ino || start.dev !== original.dev ||
        start.size > limit || relative(path, realpathSync(path))) blocked('input-changed');
    const bytes = Buffer.alloc(start.size);
    let offset = 0;
    while (offset < bytes.length) {
      const count = readSync(fd, bytes, offset, bytes.length - offset, null);
      if (!count) blocked('input-changed');
      offset += count;
    }
    const end = fstatSync(fd);
    if (end.size !== start.size || end.ctimeMs !== start.ctimeMs || end.mtimeMs !== start.mtimeMs) blocked('input-changed');
    return bytes;
  } finally {
    closeSync(fd);
  }
}

function json(bytes, reject, reason) {
  try {
    return JSON.parse(new TextDecoder('utf-8', { fatal: true }).decode(bytes));
  } catch {
    reject(reason);
  }
}

function parseGlb(bytes) {
  if (bytes.length < 28 || bytes.length % 4 || bytes.readUInt32LE(0) !== 0x46546c67 ||
      bytes.readUInt32LE(4) !== 2 || bytes.readUInt32LE(8) !== bytes.length) invalid('invalid-glb-header');
  const jsonSize = bytes.readUInt32LE(12);
  const binStart = 20 + jsonSize;
  if (!jsonSize || jsonSize % 4 || binStart + 8 > bytes.length ||
      bytes.readUInt32LE(16) !== 0x4e4f534a || bytes.readUInt32LE(binStart + 4) !== 0x004e4942 ||
      bytes.readUInt32LE(binStart) !== bytes.length - binStart - 8) invalid('invalid-glb-chunks');
  const document = json(bytes.subarray(20, binStart), invalid, 'invalid-gltf-json');
  if (!isObject(document) || document.asset?.version !== '2.0') invalid('invalid-gltf-version');
  return { document, binBytes: bytes.length - binStart - 8 };
}

function getAt(items, index) {
  if (!Array.isArray(items) || !uint(index) || !isObject(items[index])) invalid('invalid-gltf-reference');
  return items[index];
}

function readStructure(doc, binBytes) {
  if (!Array.isArray(doc.buffers) || doc.buffers.length !== 1 || !Array.isArray(doc.bufferViews)) {
    invalid('expected-embedded-buffer');
  }
  const buffer = getAt(doc.buffers, 0);
  if (Object.hasOwn(buffer, 'uri') || !positive(buffer.byteLength) ||
      buffer.byteLength > binBytes || binBytes - buffer.byteLength > 3) invalid('invalid-embedded-buffer');
  for (const view of doc.bufferViews) {
    if (!isObject(view)) invalid('invalid-buffer-view');
    const offset = defined(view, 'byteOffset', 0);
    if (view.buffer !== 0 || !uint(offset) || !positive(view.byteLength) ||
        offset > buffer.byteLength - view.byteLength) invalid('invalid-buffer-view');
    if (Object.hasOwn(view, 'byteStride') && (!uint(view.byteStride) || view.byteStride < 4 ||
        view.byteStride > 252 || view.byteStride % 4)) invalid('invalid-buffer-view-stride');
    if (Object.hasOwn(view, 'extensions')) blocked('unsupported-buffer-view');
  }
  const images = defined(doc, 'images', []);
  if (!Array.isArray(images)) invalid('invalid-gltf-image');
  for (const image of images) {
    if (!isObject(image) || Object.hasOwn(image, 'uri') ||
        !['image/png', 'image/jpeg'].includes(image.mimeType)) invalid('expected-embedded-image');
    getAt(doc.bufferViews, image.bufferView);
  }
  const extensionsUsed = defined(doc, 'extensionsUsed', []);
  const extensionsRequired = defined(doc, 'extensionsRequired', []);
  if (!Array.isArray(extensionsUsed) || !Array.isArray(extensionsRequired)) invalid('invalid-extensions');
  if (extensionsRequired.length || extensionsUsed.some(name => name !== 'KHR_materials_unlit') ||
      Object.hasOwn(doc, 'skins') || Object.hasOwn(doc, 'animations')) blocked('unsupported-gltf-profile');
}

function readAccessor(doc, index, type, sizes) {
  const accessor = getAt(doc.accessors, index);
  if (Object.hasOwn(accessor, 'sparse')) blocked('unsupported-sparse-accessor');
  const size = sizes.get(accessor.componentType);
  const view = getAt(doc.bufferViews, accessor.bufferView);
  const offset = defined(accessor, 'byteOffset', 0);
  const stride = defined(view, 'byteStride', size);
  const alignment = type === 'VEC3' ? 4 : size;
  if (accessor.type !== type || !size || !positive(accessor.count) || !uint(offset) || !positive(stride) ||
      stride < size || offset > view.byteLength - size || offset % alignment ||
      (defined(view, 'byteOffset', 0) + offset) % alignment ||
      (type === 'SCALAR' && Object.hasOwn(view, 'byteStride')) ||
      accessor.count - 1 > Math.floor((view.byteLength - offset - size) / stride)) invalid('invalid-geometry-accessor');
  return accessor.count;
}

function countMesh(doc, mesh) {
  if (!Array.isArray(mesh.primitives) || !mesh.primitives.length) invalid('missing-mesh-primitives');
  if (Object.hasOwn(mesh, 'weights')) blocked('unsupported-primitive');
  let triangles = 0;
  for (const primitive of mesh.primitives) {
    if (!isObject(primitive)) invalid('invalid-primitive');
    const mode = defined(primitive, 'mode', 4);
    if (!uint(mode) || mode > 6) invalid('invalid-primitive-mode');
    if (mode !== 4 || Object.hasOwn(primitive, 'targets') || primitive.extensions) blocked('unsupported-primitive');
    const vertices = readAccessor(doc, primitive.attributes?.POSITION, 'VEC3', vectorSizes);
    if (Object.hasOwn(primitive.attributes, 'NORMAL') &&
        readAccessor(doc, primitive.attributes.NORMAL, 'VEC3', vectorSizes) !== vertices) invalid('normal-count-mismatch');
    const count = Object.hasOwn(primitive, 'indices')
      ? readAccessor(doc, primitive.indices, 'SCALAR', indexSizes)
      : vertices;
    if (count % 3) invalid('incomplete-triangle');
    if (Object.hasOwn(primitive, 'material')) getAt(doc.materials, primitive.material);
    triangles += count / 3;
  }
  return { triangles, primitiveInstances: mesh.primitives.length };
}

function checkScene(doc, expectation) {
  const scene = getAt(doc.scenes, defined(doc, 'scene', 0));
  if (!Array.isArray(scene.nodes) || !scene.nodes.length) invalid('missing-scene-nodes');
  const pending = [...scene.nodes];
  const visited = new Set();
  const names = new Map(expectation.requiredNodes.map(name => [name, 0]));
  const meshes = new Map();
  let triangles = 0, meshNodes = 0, primitiveInstances = 0;
  while (pending.length) {
    const index = pending.pop();
    const node = getAt(doc.nodes, index);
    if (visited.has(index)) invalid('scene-is-not-a-tree');
    visited.add(index);
    if (names.has(node.name)) names.set(node.name, names.get(node.name) + 1);
    if (node.extensions || Object.hasOwn(node, 'skin') || Object.hasOwn(node, 'weights')) blocked('unsupported-node');
    for (const [key, length] of [['matrix', 16], ['translation', 3], ['rotation', 4], ['scale', 3]]) {
      if (!Object.hasOwn(node, key)) continue;
      const numbers = node[key];
      if (!Array.isArray(numbers) || numbers.length !== length || !numbers.every(Number.isFinite) ||
          (key !== 'matrix' && Object.hasOwn(node, 'matrix'))) invalid('invalid-node-transform');
    }
    if (Object.hasOwn(node, 'children')) {
      if (!Array.isArray(node.children)) invalid('invalid-node-children');
      for (const child of node.children) pending.push(child);
    }
    if (!Object.hasOwn(node, 'mesh')) continue;
    if (!meshes.has(node.mesh)) meshes.set(node.mesh, countMesh(doc, getAt(doc.meshes, node.mesh)));
    const measured = meshes.get(node.mesh);
    meshNodes++;
    triangles += measured.triangles;
    primitiveInstances += measured.primitiveInstances;
    if (triangles > expectation.budgets.triangles || primitiveInstances > expectation.budgets.primitiveInstances) {
      invalid('geometry-budget-exceeded');
    }
  }
  if (!meshNodes) invalid('missing-scene-meshes');
  if ([...names.values()].some(count => count !== 1)) invalid('required-node-missing-or-ambiguous');
  return { triangles, meshNodes, primitiveInstances };
}

function measure(args) {
  const [path, expectationPath, pin] = argumentsFor(args);
  const root = approvedRoot(path);
  const bytes = readPinnedInput(root, expectationPath, maximumExpectationBytes);
  if (digest(bytes) !== pin) blocked('expectation-pin-mismatch');
  const expected = json(bytes, blocked, 'invalid-expectation');
  if (!hasFields(expected, ['schemaVersion', 'path', 'sha256', 'budgets', 'requiredNodes']) ||
      expected.schemaVersion !== 1 || typeof expected.path !== 'string' || !expected.path.endsWith('.glb') ||
      !isDigest(expected.sha256) || !hasFields(expected.budgets, ['bytes', 'triangles', 'primitiveInstances']) ||
      !Object.values(expected.budgets).every(positive) || expected.budgets.bytes > maximumGlbBytes ||
      !Array.isArray(expected.requiredNodes) || expected.requiredNodes.length > 128 ||
      expected.requiredNodes.some(name => typeof name !== 'string' || !name.trim() || name.length > 128) ||
      new Set(expected.requiredNodes).size !== expected.requiredNodes.length) blocked('invalid-expectation');
  const glb = readPinnedInput(root, expected.path, maximumGlbBytes);
  const sha256 = digest(glb);
  if (sha256 !== expected.sha256) invalid('asset-pin-mismatch');
  if (glb.length > expected.budgets.bytes) invalid('file-budget-exceeded');
  const { document, binBytes } = parseGlb(glb);
  readStructure(document, binBytes);
  return { sha256, bytes: glb.length, ...checkScene(document, expected) };
}

const report = { schemaVersion: 1, scope: 'static-glb-intake', status: 'BLOCKED', reason: '', metrics: null };
try {
  report.metrics = measure(process.argv.slice(2));
  report.status = 'PASS';
  report.reason = 'reviewed-static-profile';
} catch (error) {
  if (error instanceof GateResult) report.status = error.status;
  report.reason = error instanceof GateResult ? error.message : 'input-unavailable';
}
process.stdout.write(JSON.stringify(report) + '\n');
process.exitCode = { PASS: 0, FAIL: 1, BLOCKED: 3 }[report.status];
