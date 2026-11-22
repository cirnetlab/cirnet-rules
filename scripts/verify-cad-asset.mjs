import { createHash } from 'node:crypto';
import { closeSync, constants, fstatSync, lstatSync, openSync, readSync, realpathSync } from 'node:fs';
import { isAbsolute, join, parse, relative, resolve, sep } from 'node:path';

const maximumGlbBytes = 32 * 1024 * 1024;
const maximumExpectationBytes = 32 * 1024;
const componentSizes = new Map([[5120, 1], [5121, 1], [5122, 2], [5123, 2], [5125, 4], [5126, 4]]);
const componentCounts = new Map([['SCALAR', 1], ['VEC2', 2], ['VEC3', 3], ['VEC4', 4]]);
const unlitExtension = 'KHR_materials_unlit';
const imageSignatures = new Map([
  ['image/png', Buffer.from('89504e470d0a1a0a', 'hex')],
  ['image/jpeg', Buffer.from('ffd8ff', 'hex')],
]);
const samplerValues = {
  magFilter: [9728, 9729],
  minFilter: [9728, 9729, 9984, 9985, 9986, 9987],
  wrapS: [33071, 33648, 10497],
  wrapT: [33071, 33648, 10497],
};
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
  return { document, binary: bytes.subarray(binStart + 8) };
}

function getAt(items, index) {
  if (!Array.isArray(items) || !uint(index) || !isObject(items[index])) invalid('invalid-gltf-reference');
  return items[index];
}

function readStructure(doc, binary) {
  if (!Array.isArray(doc.buffers) || doc.buffers.length !== 1 || !Array.isArray(doc.bufferViews)) {
    invalid('expected-embedded-buffer');
  }
  const buffer = getAt(doc.buffers, 0);
  if (Object.hasOwn(buffer, 'uri') || !positive(buffer.byteLength) ||
      buffer.byteLength > binary.length || binary.length - buffer.byteLength > 3) invalid('invalid-embedded-buffer');
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
  const imageViews = new Map();
  for (const image of images) {
    if (!isObject(image) || Object.hasOwn(image, 'uri') ||
        !imageSignatures.has(image.mimeType)) invalid('expected-embedded-image');
    checkCoreTextureExtensions(image);
    const view = getAt(doc.bufferViews, image.bufferView);
    if (Object.hasOwn(view, 'target') || Object.hasOwn(view, 'byteStride')) invalid('invalid-image-buffer-view');
    const offset = defined(view, 'byteOffset', 0);
    const signature = imageSignatures.get(image.mimeType);
    if (view.byteLength < signature.length ||
        !binary.subarray(offset, offset + signature.length).equals(signature)) invalid('image-signature-mismatch');
    imageViews.set(image.bufferView, { target: 'image' });
  }
  if (Object.hasOwn(doc, 'skins') || Object.hasOwn(doc, 'animations')) blocked('unsupported-gltf-profile');
  return imageViews;
}

function readExtensions(doc) {
  const used = defined(doc, 'extensionsUsed', []);
  const required = defined(doc, 'extensionsRequired', []);
  for (const names of [used, required]) {
    if (!Array.isArray(names) || names.some(name => typeof name !== 'string' || !name.length) ||
        new Set(names).size !== names.length) invalid('invalid-extensions');
    if (names.some(name => name !== unlitExtension)) blocked('unsupported-gltf-profile');
  }
  if (required.some(name => !used.includes(name))) invalid('required-extension-not-used');
  const materials = defined(doc, 'materials', []);
  if (!Array.isArray(materials)) invalid('invalid-materials');
  let foundUnlit = false;
  for (const material of materials) {
    if (!isObject(material)) invalid('invalid-materials');
    const extensions = defined(material, 'extensions', {});
    if (!isObject(extensions)) invalid('invalid-extensions');
    if (Object.keys(extensions).some(name => name !== unlitExtension)) blocked('unsupported-material-extension');
    if (!Object.hasOwn(extensions, unlitExtension)) continue;
    const unlit = extensions[unlitExtension];
    if (!used.includes(unlitExtension) || !isObject(unlit)) invalid('invalid-unlit-material');
    if (Object.hasOwn(unlit, 'extensions')) {
      if (!isObject(unlit.extensions)) invalid('invalid-extensions');
      if (Object.keys(unlit.extensions).length) blocked('unsupported-material-extension');
    }
    foundUnlit = true;
  }
  if (used.includes(unlitExtension) && !foundUnlit) invalid('missing-unlit-material');
}

function checkCoreTextureExtensions(record) {
  const extensions = defined(record, 'extensions', {});
  if (!isObject(extensions)) invalid('invalid-extensions');
  if (Object.keys(extensions).length) blocked('unsupported-texture-extension');
}

function checkMaterialTextures(doc, index, attributes) {
  const material = getAt(doc.materials, index);
  const pbr = defined(material, 'pbrMetallicRoughness', {});
  if (!isObject(pbr)) invalid('invalid-pbr-material');
  checkCoreTextureExtensions(pbr);
  for (const [record, field] of [
    [pbr, 'baseColorTexture'], [pbr, 'metallicRoughnessTexture'], [material, 'normalTexture'],
    [material, 'occlusionTexture'], [material, 'emissiveTexture'],
  ]) {
    if (!Object.hasOwn(record, field)) continue;
    const info = record[field];
    if (!isObject(info)) invalid('invalid-texture-info');
    checkCoreTextureExtensions(info);
    const texCoord = defined(info, 'texCoord', 0);
    if (!uint(texCoord) || !Object.hasOwn(attributes, `TEXCOORD_${texCoord}`)) invalid('invalid-texture-coordinate');
    const texture = getAt(doc.textures, info.index);
    checkCoreTextureExtensions(texture);
    getAt(doc.images, texture.source);
    if (!Object.hasOwn(texture, 'sampler')) continue;
    const sampler = getAt(doc.samplers, texture.sampler);
    checkCoreTextureExtensions(sampler);
    for (const [property, values] of Object.entries(samplerValues)) {
      if (Object.hasOwn(sampler, property) && !values.includes(sampler[property])) invalid('invalid-texture-sampler');
    }
  }
}

function readAccessor(doc, index, target, context) {
  const accessor = getAt(doc.accessors, index);
  if (Object.hasOwn(accessor, 'sparse')) blocked('unsupported-sparse-accessor');
  const width = componentSizes.get(accessor.componentType);
  const components = componentCounts.get(accessor.type);
  const size = width * components;
  const isIndex = target === 34963;
  const floating = accessor.componentType === 5126;
  const view = getAt(doc.bufferViews, accessor.bufferView);
  const offset = defined(accessor, 'byteOffset', 0);
  const start = defined(view, 'byteOffset', 0) + offset;
  const stride = defined(view, 'byteStride', size);
  const normalized = defined(accessor, 'normalized', false);
  if (!size || !positive(accessor.count) || !uint(offset) || !positive(stride) || stride < size ||
      offset > view.byteLength - size || offset % (isIndex ? width : 4) || start % width ||
      (isIndex ? Object.hasOwn(view, 'byteStride') : stride % 4 !== 0) || defined(view, 'target', target) !== target ||
      typeof normalized !== 'boolean' || ((isIndex || floating) && normalized) ||
      accessor.count - 1 > Math.floor((view.byteLength - offset - size) / stride)) invalid('invalid-geometry-accessor');

  const previous = context.views.get(accessor.bufferView);
  if (previous && (previous.target !== target ||
      (!isIndex && previous.index !== index && !Object.hasOwn(view, 'byteStride')))) invalid('inconsistent-buffer-view');
  context.views.set(accessor.bufferView, { target, index });
  return { accessor, index, width, components, start, stride, floating, normalized };
}

function scanAccessor(layout, context) {
  const { accessor, index, width, components, start, stride, floating } = layout;
  if (context.scanned.has(index)) return context.scanned.get(index);
  // Distinct accessors can alias the buffer. Cap scan work independently of file bytes.
  context.remaining -= accessor.count * width * components;
  if (context.remaining < 0) blocked('geometry-scan-limit');
  const min = Array(components).fill(Infinity);
  const max = Array(components).fill(-Infinity);
  for (let i = 0, offset = start; i < accessor.count; i++, offset += stride) {
    for (let c = 0; c < components; c++) {
      const address = offset + c * width;
      const value = floating ? context.binary.readFloatLE(address) : context.binary.readUIntLE(address, width);
      if (!Number.isFinite(value)) invalid('non-finite-vertex-component');
      min[c] = Math.min(min[c], value);
      max[c] = Math.max(max[c], value);
    }
  }
  const bounds = { min, max };
  context.scanned.set(index, bounds);
  return bounds;
}

function checkBounds(layout, bounds) {
  const { accessor, components, floating } = layout;
  for (const [name, values] of Object.entries(bounds)) {
    if (!Object.hasOwn(accessor, name)) continue;
    const declared = accessor[name];
    if (!Array.isArray(declared) || declared.length !== components || declared.some((value, c) =>
      !Number.isFinite(value) || (floating ? Math.fround(value) : value) !== values[c])) invalid('inconsistent-accessor-bounds');
  }
}

function readAttribute(doc, index, semantic, context) {
  const layout = readAccessor(doc, index, 34962, context);
  const { accessor, floating, normalized } = layout;
  const unsignedNormalized = normalized && [5121, 5123].includes(accessor.componentType);
  let valid;
  if (semantic === 'POSITION' || semantic === 'NORMAL') valid = accessor.type === 'VEC3' && floating;
  else if (semantic === 'TANGENT') valid = accessor.type === 'VEC4' && floating;
  else if (/^TEXCOORD_(?:0|[1-9]\d*)$/u.test(semantic)) valid = accessor.type === 'VEC2' && (floating || unsignedNormalized);
  else if (/^COLOR_(?:0|[1-9]\d*)$/u.test(semantic)) valid = ['VEC3', 'VEC4'].includes(accessor.type) && (floating || unsignedNormalized);
  else if (semantic.startsWith('_')) valid = accessor.componentType !== 5125;
  else if (/^(JOINTS|WEIGHTS)_\d+$/u.test(semantic)) blocked('unsupported-skinning-attribute');
  if (!valid) invalid('invalid-vertex-attribute');
  if (semantic === 'POSITION' && (!Object.hasOwn(accessor, 'min') || !Object.hasOwn(accessor, 'max'))) {
    invalid('missing-position-bounds');
  }
  if (floating) {
    const bounds = scanAccessor(layout, context);
    if (semantic === 'POSITION') checkBounds(layout, bounds);
  }
  return accessor.count;
}

function countMesh(doc, mesh, context) {
  if (!Array.isArray(mesh.primitives) || !mesh.primitives.length) invalid('missing-mesh-primitives');
  if (Object.hasOwn(mesh, 'weights')) blocked('unsupported-primitive');
  let triangles = 0;
  for (const primitive of mesh.primitives) {
    if (!isObject(primitive)) invalid('invalid-primitive');
    const mode = defined(primitive, 'mode', 4);
    if (!uint(mode) || mode > 6) invalid('invalid-primitive-mode');
    if (mode !== 4 || Object.hasOwn(primitive, 'targets') || primitive.extensions) blocked('unsupported-primitive');
    const attributes = primitive.attributes;
    if (!isObject(attributes) || !Object.hasOwn(attributes, 'POSITION')) invalid('missing-position-attribute');
    const vertices = readAttribute(doc, attributes.POSITION, 'POSITION', context);
    for (const [semantic, index] of Object.entries(attributes)) {
      if (semantic !== 'POSITION' && readAttribute(doc, index, semantic, context) !== vertices) {
        invalid('attribute-count-mismatch');
      }
    }
    let count = vertices;
    if (Object.hasOwn(primitive, 'indices')) {
      const layout = readAccessor(doc, primitive.indices, 34963, context);
      if (layout.accessor.type !== 'SCALAR' || ![5121, 5123, 5125].includes(layout.accessor.componentType)) {
        invalid('invalid-index-accessor');
      }
      const bounds = scanAccessor(layout, context);
      checkBounds(layout, bounds);
      const [max] = bounds.max;
      if (max >= vertices || max === 2 ** (8 * layout.width) - 1) invalid('triangle-index-out-of-range');
      count = layout.accessor.count;
    }
    if (count % 3) invalid('incomplete-triangle');
    if (Object.hasOwn(primitive, 'material')) checkMaterialTextures(doc, primitive.material, attributes);
    triangles += count / 3;
  }
  return { triangles, primitiveInstances: mesh.primitives.length };
}

function checkScene(doc, expectation, binary, views) {
  const scene = getAt(doc.scenes, defined(doc, 'scene', 0));
  if (!Array.isArray(scene.nodes) || !scene.nodes.length) invalid('missing-scene-nodes');
  const pending = [...scene.nodes];
  const visited = new Set();
  const names = new Map(expectation.requiredNodes.map(name => [name, 0]));
  const meshes = new Map();
  const context = { binary, views, scanned: new Map(), remaining: maximumGlbBytes };
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
    if (!meshes.has(node.mesh)) meshes.set(node.mesh, countMesh(doc, getAt(doc.meshes, node.mesh), context));
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
  const { document, binary } = parseGlb(glb);
  const views = readStructure(document, binary);
  readExtensions(document);
  return { sha256, bytes: glb.length, ...checkScene(document, expected, binary, views) };
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
