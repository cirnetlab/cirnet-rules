import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { test } from 'node:test';

const cli = fileURLToPath(new URL('../../scripts/preview-owner-checks.mjs', import.meta.url));
const temporaryDirectory = realpathSync(tmpdir());

function fixture(t) {
  const root = mkdtempSync(join(temporaryDirectory, 'cirnet-owner-preview-'));
  t.after(() => {
    const resolved = realpathSync(root);
    assert.equal(resolved, root);
    assert.equal(dirname(resolved), temporaryDirectory);
    rmSync(resolved, { recursive: true });
  });
  return root;
}

function preview(root, ...args) {
  return spawnSync(process.execPath, [cli, '--root', root, ...args], {
    cwd: root,
    encoding: 'utf8',
    timeout: 10000,
  });
}

test('previews explicitly selected checks from npm, Cargo, CMake and an RTL script without executing them', t => {
  const root = fixture(t);
  const marker = 'command-executed.txt';
  const writeMarker = `require('node:fs').writeFileSync('${marker}', 'ran')`;
  const npm = {
    scripts: { test: `node -e "${writeMarker}"`, pretest: 'npm install PRIVATE_PACKAGE_BODY' },
    privateMetadata: 'PRIVATE_PACKAGE_FIELD',
  };
  const cargo = [
    '[package]',
    'name = "private-crate-field"',
    '[[bin]]',
    'name = "verify" # owner target',
    '[[test]]',
    "name = 'smoke'",
    '[dependencies]',
    'name = "PRIVATE_CARGO_FIELD"',
  ].join('\n');
  const cmake = {
    version: 6,
    include: ['PRIVATE_CMAKE_INCLUDE.json'],
    configurePresets: [{ name: 'local', cacheVariables: { TOKEN: 'PRIVATE_CMAKE_FIELD' } }],
    buildPresets: [{ name: 'base', hidden: true }, { name: 'local', configurePreset: 'local' }],
    testPresets: [{ name: 'local', configurePreset: 'local' }],
  };
  mkdirSync(join(root, 'scripts'));
  writeFileSync(join(root, 'package.json'), JSON.stringify(npm));
  writeFileSync(join(root, 'Cargo.toml'), cargo);
  writeFileSync(join(root, 'CMakePresets.json'), JSON.stringify(cmake));
  writeFileSync(join(root, 'scripts', 'simulate.mjs'), `// PRIVATE_RTL_BODY\n${writeMarker}\n`);
  const expectedSources = ['package.json', 'Cargo.toml', 'CMakePresets.json', 'scripts/simulate.mjs'];
  const original = new Map(expectedSources.map(path => [path, readFileSync(join(root, path))]));

  const result = preview(root,
    '--manifest', 'package.json', '--manifest', 'Cargo.toml', '--manifest', 'CMakePresets.json',
    '--rtl', 'scripts/simulate.mjs', '--select', 'npm:test', '--select', 'cargo:test:smoke',
    '--select', 'cmake:test:local', '--select', 'rtl:scripts/simulate.mjs');
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stderr, '');
  assert.doesNotMatch(result.stdout, /PRIVATE_|private-crate-field|command-executed/);
  assert.equal(existsSync(join(root, marker)), false);
  assert.deepEqual(readdirSync(root).sort(), ['CMakePresets.json', 'Cargo.toml', 'package.json', 'scripts']);
  assert.deepEqual(readdirSync(join(root, 'scripts')), ['simulate.mjs']);

  const report = JSON.parse(result.stdout);
  assert.equal(report.status, 'DISCOVERED');
  assert.equal(report.executionStatus, 'NOT_RUN');
  assert.equal(report.authorization, 'NOT_GRANTED_BY_PREVIEW');
  assert.deepEqual(report.sources.map(source => source.path).sort(), expectedSources.sort());
  for (const { path, sha256 } of report.sources) {
    assert.deepEqual(readFileSync(join(root, path)), original.get(path), path);
    assert.equal(sha256, createHash('sha256').update(original.get(path)).digest('hex'), path);
  }

  const expectedChecks = {
    'npm:test': 'npm',
    'npm:pretest': 'npm',
    'cargo:bin:verify': 'cargo',
    'cargo:test:smoke': 'cargo',
    'cmake:configure:local': 'cmake',
    'cmake:build:local': 'cmake',
    'cmake:test:local': 'ctest',
    'rtl:scripts/simulate.mjs': 'node',
  };
  assert.deepEqual(Object.fromEntries(report.checks.map(check => [check.id, check.tool])), expectedChecks);
  for (const check of report.checks) {
    assert.equal(check.prerequisiteStatus, 'NOT_CHECKED', check.id);
    assert.ok(check.potentialEffects.length, check.id);
    assert.equal(check.selectionReason, check.selected ? 'EXPLICIT_SELECTION' : 'NOT_SELECTED');
  }
  assert.deepEqual(report.checks.filter(check => check.selected).map(check => check.id).sort(),
    ['cargo:test:smoke', 'cmake:test:local', 'npm:test', 'rtl:scripts/simulate.mjs']);
  assert.equal(report.sources.find(source => source.path === 'CMakePresets.json').referencesPending, true);
});

test('previews visible CMake package presets as CPack checks', t => {
  const root = fixture(t);
  writeFileSync(join(root, 'CMakePresets.json'), JSON.stringify({
    version: 6,
    configurePresets: [{ name: 'local', binaryDir: '${sourceDir}/build' }],
    packagePresets: [
      { name: 'base', hidden: true, generators: ['ZIP'] },
      { name: 'local', inherits: 'base', configurePreset: 'local' },
    ],
  }));

  const result = preview(root, '--manifest', 'CMakePresets.json', '--select', 'cmake:package:local');
  assert.equal(result.status, 0, result.stderr);
  const report = JSON.parse(result.stdout);
  assert.equal(report.executionStatus, 'NOT_RUN');
  const packagePresets = report.checks.filter(check => check.id.startsWith('cmake:package:'));
  assert.deepEqual(packagePresets.map(({ id, tool, selected }) => ({ id, tool, selected })), [
    { id: 'cmake:package:local', tool: 'cpack', selected: true },
  ]);
  assert.ok(packagePresets[0].potentialEffects.some(effect => effect.includes('packages')));
});

test('reports preset references in local CMake workflow steps', t => {
  const root = fixture(t);
  writeFileSync(join(root, 'CMakePresets.json'), JSON.stringify({
    version: 6,
    configurePresets: [{ name: 'local', binaryDir: '${sourceDir}/build' }],
    workflowPresets: [{ name: 'local', steps: [{ type: 'configure', name: 'local' }] }],
  }));

  const result = preview(root, '--manifest', 'CMakePresets.json', '--select', 'cmake:workflow:local');
  assert.equal(result.status, 0, result.stderr);
  assert.equal(JSON.parse(result.stdout).sources[0].referencesPending, true);
});

test('rejects traversal, unrecognized source types, oversized inputs and unknown check selections', t => {
  const root = fixture(t);
  writeFileSync(join(root, 'package.json'), JSON.stringify({ scripts: { test: 'tool test' } }));
  writeFileSync(join(root, 'README.md'), 'No declared manifest.');
  writeFileSync(join(root, 'CMakeLists.txt'), 'project(OwnerFixture LANGUAGES CXX)');
  writeFileSync(join(root, 'check_rtl.py'), 'raise RuntimeError("must not run")');
  mkdirSync(join(root, 'large'));
  writeFileSync(join(root, 'large', 'package.json'), ' '.repeat(256 * 1024));
  mkdirSync(join(root, 'invalid'));
  writeFileSync(join(root, 'invalid', 'package.json'), '{PRIVATE_BAD_JSON');
  const attempts = [
    ['--manifest', '../package.json'],
    ['--manifest', join(root, 'package.json')],
    ['--manifest', 'README.md'],
    ['--manifest', 'CMakeLists.txt'],
    ['--rtl', 'check_rtl.py'],
    ['--manifest', 'large/package.json'],
    ['--manifest', 'invalid/package.json'],
    ['--manifest', 'package.json', '--manifest', 'package.json'],
    ['--manifest', 'package.json', '--select', 'PRIVATE_UNKNOWN_SELECTION'],
  ];
  for (const args of attempts) {
    const result = preview(root, ...args);
    assert.equal(result.status, 2, result.stderr);
    assert.equal(result.stdout, '');
    assert.doesNotMatch(result.stderr, /PRIVATE_|invalid\/package\.json/);
  }
});

test('rejects a manifest under an indirect directory link', t => {
  const root = fixture(t);
  const destination = fixture(t);
  writeFileSync(join(destination, 'package.json'), JSON.stringify({ scripts: { private: 'PRIVATE_OUTSIDE_ROOT' } }));
  try {
    symlinkSync(destination, join(root, 'indirect'), process.platform === 'win32' ? 'junction' : 'dir');
  } catch (error) {
    if (['EPERM', 'EACCES', 'ENOSYS', 'ENOTSUP'].includes(error.code)) {
      t.skip('The host does not permit creating the directory link fixture.');
      return;
    }
    throw error;
  }
  const result = preview(root, '--manifest', 'indirect/package.json');
  assert.equal(result.status, 2, result.stderr);
  assert.equal(result.stdout, '');
  assert.doesNotMatch(result.stderr, /PRIVATE_OUTSIDE_ROOT/);
});
