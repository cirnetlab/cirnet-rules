import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { test } from 'node:test';

const source = path => new URL(`../../${path}`, import.meta.url);
const read = path => readFileSync(source(path), 'utf8');
const refactoring = 'skills/cirnet-refactoring/SKILL.md';
const focusedSkills = [
  'cirnet-task-continuity',
  'cirnet-plan-spec-authoring',
  'cirnet-candidate-evaluation',
  'cirnet-cad-engineering',
  'cirnet-engineering-workspace-ui',
  'cirnet-repository-structure',
];
const requiredSkills = [
  'cirnet-api-compatibility',
  'cirnet-code-review',
  'cirnet-contract-change',
  'cirnet-cross-repo-compatibility',
  'cirnet-debugging',
  'cirnet-documentation-sync',
  'cirnet-implementation',
  'cirnet-refactoring',
  'cirnet-regression-testing',
  'cirnet-release-changelog',
  'cirnet-verification',
  ...focusedSkills,
];
const pairs = [
  refactoring,
  ...focusedSkills.map(name => `skills/${name}/SKILL.md`),
  'rules/cad-engineering.md',
  'rules/engineering-workspace-ui.md',
  'rules/repository-organization.md',
  'skills/README.md',
  'docs/changelog.md',
  'docs/context.md',
  'docs/behavioral-evaluation.md',
  'docs/model-compatibility.md',
  'docs/setup.md',
  'plans/cirnet-rules/adaptive-stable/plan.md',
  'plans/cirnet-rules/cad-engineering/plan.md',
  'specs/cirnet-rules/adaptive-stable/spec.md',
  'specs/cirnet-rules/cad-engineering/spec.md',
];

test('Vietnamese footer identifies the exact English source bytes', () => {
  for (const path of pairs) {
    const digest = createHash('sha256').update(readFileSync(source(`en/${path}`))).digest('hex');
    assert.ok(read(`vi/${path}`).endsWith(
      `<!-- source: en/${path} -->\n<!-- source-sha256: ${digest} -->\n`,
    ), path);
  }
});

test('both languages expose the required Skills through matching fallback indexes', () => {
  const skillNames = lang => readdirSync(source(`${lang}/skills`), { withFileTypes: true })
    .filter(entry => entry.isDirectory() && existsSync(source(`${lang}/skills/${entry.name}/SKILL.md`)))
    .map(entry => entry.name).sort();
  const skills = skillNames('en');
  assert.deepEqual(skills, [...requiredSkills].sort());
  assert.deepEqual(skillNames('vi'), skills);
  for (const lang of ['en', 'vi']) {
    const links = read(`${lang}/skills/README.md`).matchAll(/\]\((?:\.\/)?([^/\s)]+)\/SKILL\.md\)/g);
    assert.deepEqual(Array.from(links, match => match[1]).sort(), skills, `${lang} fallback index`);
  }
});

test('the focused Skills have distinct selection metadata in both languages', () => {
  for (const lang of ['en', 'vi']) {
    const descriptions = new Set();
    for (const name of focusedSkills) {
      const path = `${lang}/skills/${name}/SKILL.md`;
      const frontmatter = read(path).match(/^---\n([\s\S]*?)\n---(?:\n|$)/);
      assert.ok(frontmatter, `${path}: missing frontmatter`);
      const field = key => {
        const values = [...frontmatter[1].matchAll(new RegExp(`^${key}:[ \\t]*(.*)$`, 'gm'))];
        assert.equal(values.length, 1, `${path}: expected one ${key} field`);
        return values[0][1].trim().replace(/^'(.*)'$/, '$1').trim();
      };
      assert.equal(field('name'), name, path);
      const description = field('description');
      assert.ok(description && [...description].length <= 1024, `${path}: invalid description`);
      assert.ok(!descriptions.has(description), `${path}: duplicate selection description`);
      descriptions.add(description);
    }
  }
});

test('both languages retain the complete Rules and non-prompt template inventories', () => {
  const rules = ['cad-engineering.md', 'coding.md', 'cross-repo.md', 'engineering-workspace-ui.md',
    'repository-organization.md', 'security.md', 'testing.md'];
  const templates = ['AGENTS.md', 'compatibility-matrix.md', 'plan.md', 'repository-map.md', 'spec.md'];
  for (const lang of ['en', 'vi']) {
    assert.deepEqual(readdirSync(source(`${lang}/rules`)).sort(), rules);
    assert.deepEqual(readdirSync(source(`${lang}/templates`)).sort(), templates);
    for (const guide of ['docs/context.md', 'docs/behavioral-evaluation.md']) {
      assert.match(read(`${lang}/${guide}`), /model-compatibility\.md/);
    }
  }
});

test('refactoring preserves contracted error behavior and reports incidental diagnostics', () => {
  const skill = read(`en/${refactoring}`);
  assert.match(skill, /Preserve error types, codes and documented message text/);
  assert.match(skill, /Report incidental runtime diagnostic wording changes without expanding the edit/);
});

test('refactoring checks the separators left behind by a removed declaration', () => {
  const skill = read(`en/${refactoring}`);
  assert.match(skill, /remove the blank separator that belonged only to the declaration/);
  assert.match(skill, /review the scoped diff including whitespace/i);
});
