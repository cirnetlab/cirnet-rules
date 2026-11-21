import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, readFileSync, readdirSync } from 'node:fs';
import { test } from 'node:test';

const source = path => new URL(`../../${path}`, import.meta.url);
const read = path => readFileSync(source(path), 'utf8');
const authoring = 'skills/cirnet-work-prompt-authoring/SKILL.md';
const refactoring = 'skills/cirnet-refactoring/SKILL.md';
const template = 'templates/chat-to-work-prompt.md';
const focusedSkills = [
  'cirnet-task-continuity',
  'cirnet-plan-spec-authoring',
  'cirnet-candidate-evaluation',
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
  'cirnet-work-prompt-authoring',
  ...focusedSkills,
];
const pairs = [
  authoring,
  refactoring,
  template,
  ...focusedSkills.map(name => `skills/${name}/SKILL.md`),
  'rules/engineering-workspace-ui.md',
  'rules/repository-organization.md',
  'skills/README.md',
  'docs/changelog.md',
  'docs/context.md',
  'docs/behavioral-evaluation.md',
  'docs/model-compatibility.md',
  'docs/setup.md',
  'plans/cirnet-rules/adaptive-stable/plan.md',
  'specs/cirnet-rules/adaptive-stable/spec.md',
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
  for (const name of requiredSkills) assert.ok(skills.includes(name), `Missing Skill: ${name}`);
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

test('both languages keep one Work Prompt template', () => {
  for (const lang of ['en', 'vi']) {
    const templates = readdirSync(source(`${lang}/templates`)).filter(name => name.includes('work-prompt'));
    assert.deepEqual(templates, ['chat-to-work-prompt.md']);
    for (const mode of ['LIGHT', 'STANDARD', 'DEEP']) assert.ok(read(`${lang}/${template}`).includes(mode));
  }
});

test('the canonical authoring pair and evaluation guides route model changes to the compatibility procedure', () => {
  for (const lang of ['en', 'vi']) {
    const term = lang === 'en' ? /model-agnostic/i : /độc lập model/i;
    for (const path of [authoring, template]) assert.match(read(`${lang}/${path}`), term);
    for (const path of [authoring, template, 'docs/context.md', 'docs/behavioral-evaluation.md']) {
      assert.match(read(`${lang}/${path}`), /model-compatibility\.md/, `${lang}/${path}`);
    }
  }
});

test('Author stops discovery at a safe handoff and preserves boundaries on read failure', () => {
  const skill = read(`en/${authoring}`);
  const guide = read(`en/${template}`);
  assert.match(skill, /Draft the goal and granted scope before discovery/);
  assert.match(skill, /Stop discovery once you can hand off safely/);
  assert.match(guide, /Only retry with a verified\s+alternative[\s\S]*same read scope and output limits/);
  assert.match(guide, /Honor access and\s+shell-policy denials/);
  assert.match(guide, /`NOT_INSPECTED`[\s\S]*scoped Work preflight/);
  assert.match(guide, /Only block the handoff when an essential decision, authority, conflict or\s+safety condition/);
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
