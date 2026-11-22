import { createHash } from 'node:crypto';
import { closeSync, constants, fstatSync, lstatSync, openSync, readSync, realpathSync } from 'node:fs';
import { basename, extname, isAbsolute, join, parse, relative, resolve, sep } from 'node:path';

const maximumBytes = 128 * 1024;
const maximumInputs = 16;
const maximumChecks = 256;
const identifier = /^[A-Za-z0-9][A-Za-z0-9_.:-]{0,63}$/u;
const usage = 'Usage: node preview-owner-checks.mjs --root <absolute-local-operator-approved-root> ' +
  '[--manifest <relative-package.json|Cargo.toml|CMakePresets.json>] ' +
  '[--rtl <relative-owner-declared-script>] [--select <candidate-id>]';

class PreviewError extends Error {}

function parseArguments(args) {
  if (args.length % 2 !== 0 || args.length > 2 * (1 + maximumInputs + maximumChecks)) {
    throw new PreviewError(usage);
  }
  const request = { inputs: [], selected: new Set() };
  for (let index = 0; index < args.length; index += 2) {
    const [flag, value] = args.slice(index, index + 2);
    if (!value || value.startsWith('--')) throw new PreviewError(usage);
    switch (flag) {
      case '--root':
        if (request.root) throw new PreviewError('Specify a single approved root.');
        request.root = value;
        break;
      case '--manifest':
      case '--rtl':
        request.inputs.push({ path: value, rtl: flag === '--rtl' });
        break;
      case '--select':
        if (request.selected.has(value)) throw new PreviewError('Duplicate candidate selection.');
        request.selected.add(value);
        break;
      default:
        throw new PreviewError(usage);
    }
  }
  if (!request.root || !request.inputs.length || request.inputs.length > maximumInputs) {
    throw new PreviewError(usage);
  }
  return request;
}

function approvedRoot(path) {
  if (!isAbsolute(path) || /^[\\/]{2}/u.test(path) || path.split(/[\\/]/u).includes('..')) {
    throw new PreviewError('Use an absolute local root without traversal.');
  }
  const root = resolve(path);
  let directory = parse(root).root;
  const withinVolume = relative(directory, root);
  if (!withinVolume) throw new PreviewError('Choose an owned directory below the filesystem root.');
  for (const component of withinVolume.split(sep)) {
    directory = join(directory, component);
    if (!lstatSync(directory).isDirectory()) {
      throw new PreviewError('Root directories must exist and must be regular, unlinked directories.');
    }
  }
  if (relative(root, realpathSync(root))) throw new PreviewError('The selected root resolves through a link.');
  return root;
}

function readInput(root, name) {
  const parts = name.split('/');
  if (name.length > 256 || parts.some(part => !/^[A-Za-z0-9_.-]+$/u.test(part) ||
      part === '.' || part === '..' || part.endsWith('.') ||
      /^(?:con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/iu.test(part))) {
    throw new PreviewError('Use a portable relative file path without links or traversal.');
  }
  let directory = root;
  for (const component of parts.slice(0, -1)) {
    directory = join(directory, component);
    if (!lstatSync(directory).isDirectory()) throw new PreviewError('Linked or unsupported input directory.');
  }
  const path = join(directory, parts.at(-1));
  const original = lstatSync(path);
  if (!original.isFile() || original.nlink !== 1 || original.size > maximumBytes) {
    throw new PreviewError('Use one regular, unlinked input file of at most 128 KiB.');
  }
  const descriptor = openSync(path, constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0) | (constants.O_NONBLOCK ?? 0));
  try {
    const opened = fstatSync(descriptor);
    if (!opened.isFile() || opened.nlink !== 1 || opened.size > maximumBytes ||
        opened.ino !== original.ino || opened.dev !== original.dev || relative(path, realpathSync(path))) {
      throw new PreviewError('Input changed or resolves through a link.');
    }
    const buffer = Buffer.alloc(maximumBytes + 1);
    let length = 0;
    while (length < buffer.length) {
      const count = readSync(descriptor, buffer, length, buffer.length - length, null);
      if (!count) break;
      length += count;
    }
    const finished = fstatSync(descriptor);
    if (length > maximumBytes || length !== opened.size ||
        finished.mtimeMs !== opened.mtimeMs || finished.ctimeMs !== opened.ctimeMs) {
      throw new PreviewError('Input changed or exceeds the preview size limit.');
    }
    return buffer.subarray(0, length);
  } finally {
    closeSync(descriptor);
  }
}

function candidate(id, tool, additionalEffect) {
  return {
    id,
    tool,
    prerequisiteStatus: 'NOT_CHECKED',
    potentialEffects: ['owner code may write files or access external resources', additionalEffect],
  };
}

function assertNamed(label) {
  if (typeof label !== 'string' || !identifier.test(label)) {
    throw new PreviewError('An input declares an unsupported candidate identifier.');
  }
}

function objectFromJson(source) {
  let data;
  try {
    data = JSON.parse(source);
  } catch {
    throw new PreviewError('A selected JSON manifest is invalid.');
  }
  if (!data || typeof data !== 'object' || Array.isArray(data)) {
    throw new PreviewError('Select a JSON manifest containing an object.');
  }
  return data;
}

function npmChecks(source) {
  const { scripts = {} } = objectFromJson(source);
  if (!scripts || typeof scripts !== 'object' || Array.isArray(scripts)) {
    throw new PreviewError('Expected a package.json scripts object.');
  }
  return Object.entries(scripts).map(([name, body]) => {
    assertNamed(name);
    if (typeof body !== 'string') throw new PreviewError('Expected an npm script string.');
    return candidate('npm:' + name, 'npm', 'may invoke npm lifecycle hooks');
  });
}

function cargoChecks(source) {
  if (source.includes('"""') || source.includes("'''")) {
    throw new PreviewError('Cargo discovery supports explicit targets and single-line strings.');
  }
  const checks = [];
  let kind;
  let named;
  for (const raw of source.split(/\r?\n/u)) {
    const line = raw.trim();
    if (line.startsWith('[')) {
      const header = /^(?:\[(lib)\]|\[\[(bin|test|example|bench)\]\])\s*(?:#.*)?$/u.exec(line);
      kind = header && (header[1] ?? header[2]);
      named = false;
    } else if (kind && /^name\s*=/u.test(line)) {
      const match = /^name\s*=\s*(["'])([A-Za-z0-9][A-Za-z0-9_.-]{0,63})\1\s*(?:#.*)?$/u.exec(line);
      if (!match || named) throw new PreviewError('Expected one literal name per explicit Cargo target.');
      checks.push(candidate('cargo:' + kind + ':' + match[2], 'cargo', 'may build artifacts or invoke build scripts'));
      named = true;
    }
  }
  return checks;
}

function cmakeChecks(source) {
  const data = objectFromJson(source);
  if (!Number.isSafeInteger(data.version) || data.version < 1) {
    throw new PreviewError('Expected a CMake preset document with a numeric version.');
  }
  const checks = [];
  let referencesPending = Boolean(data.include?.length);
  const tools = { configure: 'cmake', build: 'cmake', test: 'ctest', package: 'cpack', workflow: 'cmake' };
  for (const [kind, tool] of Object.entries(tools)) {
    const presets = data[kind + 'Presets'] ?? [];
    if (!Array.isArray(presets)) throw new PreviewError('Expected CMake preset arrays.');
    for (const preset of presets) {
      assertNamed(preset?.name);
      referencesPending ||= Boolean(preset.inherits || preset.configurePreset ||
        (kind === 'workflow' && preset.steps?.length));
      if (preset.hidden === true) continue;
      const effect = kind === 'package' ? 'may create packages or run packaging scripts' :
        'may configure or build artifacts, run tests or start dependent preset steps';
      checks.push(candidate('cmake:' + kind + ':' + preset.name, tool, effect));
    }
  }
  return { checks, referencesPending };
}

function inspect(root, input) {
  const format = input.rtl ? 'rtl' : new Map([
    ['package.json', 'npm'],
    ['Cargo.toml', 'cargo'],
    ['CMakePresets.json', 'cmake'],
  ]).get(basename(input.path));
  if (!format) throw new PreviewError('Select package.json, Cargo.toml, CMakePresets.json, or an explicit RTL script.');
  const tool = input.rtl && new Map([
    ['.js', 'node'], ['.mjs', 'node'], ['.ps1', 'pwsh'], ['.sh', 'sh'], ['.bat', 'cmd'], ['.cmd', 'cmd'],
  ]).get(extname(input.path));
  if (input.rtl && !tool) throw new PreviewError('Use an explicitly named RTL script with a supported extension.');

  const bytes = readInput(root, input.path);
  const source = { path: input.path, format, sha256: createHash('sha256').update(bytes).digest('hex') };
  if (input.rtl) {
    return { source, checks: [candidate('rtl:' + input.path, tool, 'may invoke an RTL compiler or simulator')] };
  }
  const text = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
  if (format === 'npm') return { source, checks: npmChecks(text) };
  if (format === 'cargo') return { source, checks: cargoChecks(text) };
  const { checks, referencesPending } = cmakeChecks(text);
  return { source: { ...source, referencesPending }, checks };
}

function run(args) {
  const request = parseArguments(args);
  const root = approvedRoot(request.root);
  const sources = [];
  const checks = [];
  const sourcePaths = new Set();
  const candidateIds = new Set();
  for (const input of request.inputs) {
    const key = process.platform === 'win32' ? input.path.toLowerCase() : input.path;
    if (sourcePaths.has(key)) throw new PreviewError('Duplicate preview input.');
    sourcePaths.add(key);
    const preview = inspect(root, input);
    sources.push(preview.source);
    for (const check of preview.checks) {
      if (candidateIds.has(check.id) || checks.length >= maximumChecks) {
        throw new PreviewError('Preview candidates are ambiguous or exceed the inventory limit.');
      }
      candidateIds.add(check.id);
      const selected = request.selected.has(check.id);
      checks.push({
        ...check,
        source: input.path,
        selected,
        selectionReason: selected ? 'EXPLICIT_SELECTION' : 'NOT_SELECTED',
      });
    }
  }
  if ([...request.selected].some(id => !candidateIds.has(id))) {
    throw new PreviewError('An explicitly selected candidate was not discovered.');
  }
  return {
    status: checks.length ? 'DISCOVERED' : 'UNAVAILABLE',
    executionStatus: 'NOT_RUN',
    authorization: 'NOT_GRANTED_BY_PREVIEW',
    limits: [
      'Selection records operator intent. Review the owning workflow, dependencies, effects and authorization before execution.',
      'Tool availability, script bodies, Cargo implicit targets, CMake references and RTL dependencies are unverified.',
      'Local path checks reduce accidental traversal. Use a trusted workspace and external permissions for isolation.',
    ],
    sources,
    checks,
  };
}

const args = process.argv.slice(2);
if (args.length === 1 && args[0] === '--help') {
  process.stdout.write(usage + '\n');
} else {
  try {
    process.stdout.write(JSON.stringify(run(args), null, 2) + '\n');
  } catch (error) {
    // Runtime parser and filesystem diagnostics can contain raw input and private paths.
    const message = error instanceof PreviewError ? error.message : 'A selected input could not be read safely.';
    process.stderr.write('OWNER_CHECK_PREVIEW_BLOCKED: ' + message + '\n');
    process.exitCode = 2;
  }
}
